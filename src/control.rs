use crate::{
    proto::wafl::*,
    runner::{App, Runner},
    services::wafl::{RunnerService, RunnerServiceExt},
};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub fn router(runner: Runner, token: Option<String>) -> connectrpc::Router {
    RunnerServiceExt::register(
        Arc::new(Control { runner, token }),
        connectrpc::Router::new(),
    )
}

struct Control {
    runner: Runner,
    token: Option<String>,
}
impl Control {
    fn authorize(&self, ctx: &RequestContext) -> Result<(), ConnectError> {
        if let Some(token) = &self.token {
            let mut headers = ctx.headers().get_all("authorization").iter();
            let supplied = headers
                .next()
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));
            if supplied != Some(token.as_str()) || headers.next().is_some() {
                return Err(ConnectError::unauthenticated("runner token required"));
            }
        }
        Ok(())
    }
    fn app(&self, ctx: &RequestContext, name: &str) -> Result<Arc<App>, ConnectError> {
        self.authorize(ctx)?;
        self.runner
            .app(name)
            .ok_or_else(|| ConnectError::not_found("app not configured on this runner"))
    }
}
#[allow(refining_impl_trait)]
impl RunnerService for Control {
    async fn list(
        &self,
        ctx: RequestContext,
        _: ServiceRequest<'_, ListRequest>,
    ) -> ServiceResult<ListResponse> {
        self.authorize(&ctx)?;
        Response::ok(ListResponse {
            items: self.runner.list(),
            ..Default::default()
        })
    }
    async fn get(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, GetRequest>,
    ) -> ServiceResult<Instance> {
        Response::ok(self.app(&ctx, &req.to_owned_message().name)?.snapshot())
    }
    async fn start(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, StartRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&ctx, &req.to_owned_message().name)?;
        let lease = self
            .runner
            .acquire(&app)
            .await
            .map_err(|e| ConnectError::unavailable(e.to_string()))?;
        drop(lease);
        Response::ok(app.snapshot())
    }
    async fn stop(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, StopRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&ctx, &req.to_owned_message().name)?;
        app.stop(false, false)
            .await
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        Response::ok(app.snapshot())
    }
}
