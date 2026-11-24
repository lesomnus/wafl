//! Shared interceptors for generated application service layers.
use connectrpc::{CodecFormat, ConnectError, Encodable, RequestContext, Response, ServiceStream};
use futures_util::StreamExt;
use std::sync::{Arc, LazyLock};

pub trait Hook: Send + Sync + 'static {
    fn enter(&self, _: &'static str, _: &mut RequestContext) -> Result<(), ConnectError> {
        Ok(())
    }
    fn receive(&self, _: &'static str, _: &RequestContext, _: &[u8]) -> Result<(), ConnectError> {
        Ok(())
    }
    fn send(
        &self,
        _: &'static str,
        _: &RequestContext,
        _: &mut Vec<u8>,
    ) -> Result<(), ConnectError> {
        Ok(())
    }
    /// A stream hook may deliver the message to its own consumer. Return false to stop forwarding it.
    fn forward_stream(&self,_:&str,_:&RequestContext,_:&[u8])->Result<bool,ConnectError>{Ok(true)}
    fn leave(&self, _: &'static str, _: &RequestContext, _: Option<&ConnectError>) {}
}
#[derive(Clone, Default)]
pub struct Layer {
    hooks: Vec<Arc<dyn Hook>>,
    redact: bool,
}
impl Layer {
    pub fn new(hooks: Vec<Arc<dyn Hook>>) -> Self {
        Self {
            hooks,
            redact: false,
        }
    }
    pub fn public(mut self) -> Self {
        self.redact = true;
        self
    }
    pub(crate) fn enter(
        &self,
        method: &'static str,
        mut ctx: RequestContext,
    ) -> Result<Call, ConnectError> {
        let mut count = 0;
        for hook in &self.hooks {
            if let Err(error) = hook.enter(method, &mut ctx) {
                for h in self.hooks[..count].iter().rev() {
                    h.leave(method, &ctx, Some(&error));
                }
                return Err(error);
            }
            count += 1;
        }
        Ok(Call {
            layer: self.clone(),
            method,
            ctx,
            done: false,
            #[cfg(not(target_arch = "wasm32"))]
            started: Some(paydirt::slow::Timer::start()),
        })
    }
}
enum Delivery<M>{Forward(Option<M>),Consumed}
pub(crate) struct Call {
    layer: Layer,
    method: &'static str,
    pub ctx: RequestContext,
    done: bool,
    #[cfg(not(target_arch = "wasm32"))]
    started: Option<paydirt::slow::Timer>,
}
impl Call {
    fn finish(&mut self, err: Option<&ConnectError>) {
        if self.done {
            return;
        }
        self.done = true;
        for h in self.layer.hooks.iter().rev() {
            h.leave(self.method, &self.ctx, err);
        }
    }
    pub fn receive(&mut self, bytes: &[u8]) -> Result<(), ConnectError> {
        for hook in &self.layer.hooks {
            if let Err(error) = hook.receive(self.method, &self.ctx, bytes) {
                self.finish(Some(&error));
                return Err(error);
            }
        }
        Ok(())
    }
    pub fn stream_input<M: buffa::HasMessageView + Send + 'static>(
        &mut self,
        incoming: connectrpc::InboundStream<M>,
    ) -> connectrpc::InboundStream<M> {
        if self.layer.hooks.is_empty() {
            return incoming;
        }
        let hooks = self.layer.hooks.clone();
        let ctx = self.ctx.clone();
        let method = self.method;
        Box::pin(incoming.map(move |item| {
            let msg = item?;
            for hook in &hooks {
                hook.receive(method, &ctx, msg.bytes())?;
            }
            Ok(msg)
        }))
    }
    fn output<M: buffa::Message + buffa::MessageName + serde::Serialize, R: Encodable<M>>(
        &self,
        body: &R,
        stream:bool,
    ) -> Result<Delivery<M>, ConnectError> {
        static SCHEMA: LazyLock<paydirt_store::secrets::Schema> = LazyLock::new(|| {
            paydirt_store::secrets::Schema::new(crate::transport::ADMISSION_SCHEMA)
                .expect("generated field rules")
        });
        let redact = self.layer.redact && SCHEMA.protects(M::FULL_NAME);
        if self.layer.hooks.is_empty() && !redact {
            return Ok(Delivery::Forward(None));
        }
        let mut wire = body.encode(CodecFormat::Proto)?.to_vec();
        for h in self.layer.hooks.iter().rev() {
            h.send(self.method, &self.ctx, &mut wire)?;
            if stream && !h.forward_stream(self.method,&self.ctx,&wire)?{return Ok(Delivery::Consumed)}
        }
        if redact {
            wire = SCHEMA
                .clean(M::FULL_NAME, &wire)
                .map_err(crate::transport::error)?;
        }
        Ok(Delivery::Forward(Some(M::decode_from_slice(&wire).map_err(|_| {
            ConnectError::internal("invalid interceptor response")
        })?)))
    }
    pub fn unary<M: buffa::Message + buffa::MessageName + serde::Serialize, R: Encodable<M>>(
        mut self,
        res: Result<Response<R>, ConnectError>,
    ) -> Result<Response<Body<M, R>>, ConnectError> {
        let res = res.and_then(|reply| {
            let Delivery::Forward(public)=self.output(&reply.body,false)? else {return Err(ConnectError::internal("incomplete unary response"))};
            Ok(reply.map_body(|original| Body { original, public }))
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(timer) = self.started.take() {
            timer.finish(|| self.method.into());
        }
        self.finish(res.as_ref().err());
        res
    }
    pub fn stream<
        M: buffa::Message + buffa::MessageName + serde::Serialize + Send + 'static,
        R: Encodable<M> + Send + 'static,
    >(
        mut self,
        res: Result<Response<ServiceStream<R>>, ConnectError>,
    ) -> Result<Response<ServiceStream<Body<M, R>>>, ConnectError> {
        let reply = match res {
            Ok(r) => r,
            Err(e) => {
                self.finish(Some(&e));
                return Err(e);
            }
        };
        Ok(reply.map_body(move|mut incoming|{
            let stream=async_stream::stream!{
                while let Some(item)=incoming.next().await{
                    match item.and_then(|original|Ok(match self.output(&original,true)?{Delivery::Consumed=>None,Delivery::Forward(public)=>Some(Body{original,public})})){
                        Ok(Some(row))=>yield Ok(row),
                        Ok(None)=>continue,
                        Err(e)=>{self.finish(Some(&e));yield Err(e);return;}
                    }
                }
                self.finish(None);
            };
            Box::pin(stream) as ServiceStream<_>
        }))
    }
}
impl Drop for Call {
    fn drop(&mut self) {
        if !self.done {
            self.finish(Some(&ConnectError::canceled("call cancelled")));
        }
    }
}
pub(crate) struct Body<M, R> {
    original: R,
    public: Option<M>,
}
impl<M: buffa::Message + serde::Serialize, R: Encodable<M>> Encodable<M> for Body<M, R> {
    fn encode(&self, codec: CodecFormat) -> Result<buffa::bytes::Bytes, ConnectError> {
        match &self.public {
            Some(m) => Encodable::<M>::encode(m, codec),
            None => self.original.encode(codec),
        }
    }
}
