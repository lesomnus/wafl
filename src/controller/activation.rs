use super::*;
use crate::{
    config::Protocol,
    proxy::{ProxyBody, grpc_deadline, reply, upstream},
};
use http::{Request, Response, StatusCode, header};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use paydirt::spin::Stop;
use paydirt_store::Fault;
use std::convert::Infallible;
use tokio::{net::TcpListener, sync::oneshot, task::JoinSet};

struct Admission {
    id: Vec<u8>,
    done: Option<oneshot::Sender<()>>,
}
impl Drop for Admission {
    fn drop(&mut self) {
        if let Some(done) = self.done.take() {
            let _ = done.send(());
        }
    }
}
impl Control {
    async fn enqueue(&self, domain: String) -> Result<(App, Admission), ConnectError> {
        let this = self.clone();
        // This task owns the committed admission even if the client disconnects
        // while the blocking database transaction is still running.
        tokio::spawn(async move {
            let id=paydirt::mint(13).to_vec();let key=id.clone();
            let session=uuid::Uuid::new_v4().to_string();
            let budget=this.config.traffic.as_ref().unwrap().waiting;
            let app=this.db.transaction(move|tx| {
                let mut app=all::<App>(tx,"App")?.into_iter().find(|a|a.domain==domain)
                    .ok_or_else(||Fault{code:"not_found".into(),..Fault::invalid("unknown app domain")})?;
                if app.scaling.as_option().is_none() { return Err(Fault::precondition("app does not enable activation")); }
                let traffic=all::<Traffic>(tx,"Traffic")?;
                let queued:Vec<_>=traffic.iter().filter(|t|t.activation && (!t.closed || t.active>0)).collect();
                if queued.len()>=budget as usize || queued.iter().filter(|t|t.app.id==app.id).count()>=app.scaling.waiting as usize {
                    return Err(Fault::exhausted("activation queue is full"));
                }
                call(tx,"Traffic","Add",json!({"id":encode(&key),"tenant":{"id":encode(&app.tenant.id)},
                    "app":{"id":encode(&app.id)},"session":session,"revision":app.revision.to_string(),
                    "activation":true,"active":1,"started":"1","dateSeen":now().to_string()}))?;
                if app.state.desired==0 {
                    let mut state=(*app.state).clone();state.desired=1;state.date_activity=now();state.date_change=now();
                    app=decode(call(tx,"App","Patch",json!({"ref":{"id":encode(&app.id)},"state":state}))?)?;
                }
                Ok(app)
            }).await?;
            let (done,mut ended)=oneshot::channel();
            let key=id.clone();let start=tokio::time::Instant::now();
            let interval=this.config.traffic.as_ref().unwrap().interval_ms;
            tokio::spawn(async move {
                let mut closed=false;
                loop {
                    if !closed {
                        tokio::select! { _=&mut ended=>closed=true, _=tokio::time::sleep(Duration::from_millis(interval))=>{} }
                    }
                    let key=key.clone();let busy=start.elapsed().as_millis().min(u64::MAX as u128) as u64;
                    let result=this.db.transaction(move|tx| {
                        let t:Traffic=decode(call(tx,"Traffic","Get",json!({"ref":{"id":encode(&key)}}))?)?;
                        if t.closed {return Ok(());}
                        traffic::patch(tx,&key,json!({"active":u32::from(!closed),"closed":closed,"busyMs":busy.to_string(),"dateSeen":now().to_string()}))?;
                        Ok(())
                    }).await;
                    if closed && (result.is_ok() || result.as_ref().is_err_and(|e|e.code==connectrpc::ErrorCode::NotFound)) {break;}
                    if closed {tokio::time::sleep(Duration::from_millis(interval)).await;}
                }
            });
            Ok((app,Admission{id,done:Some(done)}))
        }).await.map_err(|_|ConnectError::internal("activation worker failed"))?
    }
    async fn assign(&self, admission: &[u8], app: &App) -> Result<Option<Placement>, ConnectError> {
        let key = admission.to_vec();
        let app = app.clone();
        let fresh = self.config.fresh_ms;
        self.db
            .transaction(move |tx| {
                let current: App = decode(call(
                    tx,
                    "App",
                    "Get",
                    json!({"ref":{"id":encode(&app.id)}}),
                )?)?;
                if current.revision != app.revision || current.scaling.as_option().is_none() {
                    return Err(Fault::aborted("app changed during activation"));
                }
                let t: Traffic = decode(call(
                    tx,
                    "Traffic",
                    "Get",
                    json!({"ref":{"id":encode(&key)}}),
                )?)?;
                if t.closed || t.draining {
                    return Err(Fault::precondition("activation was closed"));
                }
                let traffic = all::<Traffic>(tx, "Traffic")?;
                let mut ready: Vec<_> = all::<Placement>(tx, "Placement")?
                    .into_iter()
                    .filter(|p| {
                        p.app.id == app.id
                            && p.revision == app.revision
                            && p.phase == PlacementPhase::Ready
                            && p.managed
                            && p.date_seen <= now()
                            && now().saturating_sub(p.date_seen) <= fresh
                            && p.execution.phase == Phase::Running
                            && !p.execution.draining
                    })
                    .collect();
                ready.sort_by_key(|p| {
                    (
                        traffic
                            .iter()
                            .filter(|t| t.placement.id == p.id)
                            .map(|t| u64::from(t.active))
                            .sum::<u64>(),
                        p.id.clone(),
                    )
                });
                let Some(p) = ready.into_iter().next() else {
                    return Ok(None);
                };
                traffic::patch(
                    tx,
                    &key,
                    json!({"placement":{"id":encode(&p.id)},"dateSeen":now().to_string()}),
                )?;
                Ok(Some(p))
            })
            .await
    }
}
fn deadline_response(grpc: bool) -> Response<ProxyBody> {
    if !grpc {
        return reply(
            StatusCode::GATEWAY_TIMEOUT,
            "app startup deadline exceeded\n",
        );
    }
    let mut response = reply(StatusCode::OK, "");
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/grpc"),
    );
    response
        .headers_mut()
        .insert("grpc-status", http::HeaderValue::from_static("4"));
    response
}
async fn forward(
    controller: Control,
    request: Request<Incoming>,
) -> Result<Response<ProxyBody>, Infallible> {
    let authority = request.uri().authority().map(|a| a.as_str()).or_else(|| {
        request
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
    });
    let domain = authority
        .filter(|a| !a.contains('@'))
        .and_then(|a| a.parse::<http::uri::Authority>().ok())
        .map(|a| a.host().to_ascii_lowercase());
    let Some(domain) = domain else {
        return Ok(reply(StatusCode::BAD_REQUEST, "invalid app authority\n"));
    };
    if request.method() == http::Method::CONNECT || request.headers().contains_key(header::UPGRADE)
    {
        return Ok(reply(
            StatusCode::NOT_IMPLEMENTED,
            "protocol upgrades are not supported\n",
        ));
    }
    let grpc = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h == "application/grpc" || h.starts_with("application/grpc+"));
    let deadline = if grpc {
        match grpc_deadline(request.headers()) {
            Ok(v) => v,
            Err(()) => return Ok(reply(StatusCode::BAD_REQUEST, "invalid grpc-timeout\n")),
        }
    } else {
        None
    };
    let operation = async {
        let (app, admission) = match controller.enqueue(domain).await {
            Ok(v) => v,
            Err(e) => {
                return reply(
                    match e.code {
                        connectrpc::ErrorCode::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
                        connectrpc::ErrorCode::NotFound => StatusCode::NOT_FOUND,
                        _ => StatusCode::SERVICE_UNAVAILABLE,
                    },
                    "activation unavailable\n",
                );
            }
        };
        let ready = tokio::time::timeout(Duration::from_millis(app.scaling.wait_ms), async {
            loop {
                if let Some(p) = controller.assign(&admission.id, &app).await? {
                    return Ok::<_, ConnectError>(p);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;
        let placement = match ready {
            Ok(Ok(p)) => p,
            Ok(Err(_)) => {
                return reply(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "app changed or controller unavailable\n",
                );
            }
            Err(_) => return deadline_response(grpc),
        };
        let Ok(address) = placement.execution.endpoint.address.parse() else {
            return reply(StatusCode::BAD_GATEWAY, "invalid app endpoint\n");
        };
        let protocol = if app.manifest.protocol == "h2c" {
            Protocol::H2c
        } else {
            Protocol::Http
        };
        upstream(address, protocol, request, deadline, admission).await
    };
    Ok(if let Some(deadline) = deadline {
        tokio::select! {biased;_=tokio::time::sleep_until(deadline)=>deadline_response(grpc), response=operation=>response}
    } else {
        operation.await
    })
}
pub(super) async fn serve(
    controller: Control,
    listener: Arc<TcpListener>,
    stop: Stop,
) -> Result<(), String> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _=stop.cancelled()=>break,
            Some(_)=connections.join_next()=>{},
            socket=listener.accept()=>{
                let (socket,_)=socket.map_err(|e|e.to_string())?;
                let controller=controller.clone();let stop=stop.clone();
                connections.spawn(async move {
                    let service=hyper::service::service_fn(move|r|forward(controller.clone(),r));
                    let mut builder=hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
                    builder.http1().header_read_timeout(Duration::from_secs(10)).timer(hyper_util::rt::TokioTimer::new());
                    let connection=builder.serve_connection(TokioIo::new(socket),service);tokio::pin!(connection);
                    tokio::select! {_=&mut connection=>{},_=stop.cancelled()=>{
                        connection.as_mut().graceful_shutdown();let _=tokio::time::timeout(Duration::from_secs(30),connection).await;
                    }}
                });
            }
        }
    }
    while connections.join_next().await.is_some() {}
    Ok(())
}
