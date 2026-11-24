use crate::{
    proto::wafl::*,
    runner::{App, Runner},
    services::wafl::InstanceService,
    storage,
};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use std::sync::Arc;

pub fn router(runner: Runner, token: Option<String>) -> connectrpc::Router {
    storage::mount(connectrpc::Router::new(), &Instances { runner, token })
}

#[derive(Clone)]
struct Instances {
    runner: Runner,
    token: Option<String>,
}
impl storage::Layers for Instances {
    fn wafl_instance(&self) -> impl InstanceService {
        self.clone()
    }
}

impl Instances {
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
    fn app(&self, ctx: &RequestContext, id: &[u8]) -> Result<Arc<App>, ConnectError> {
        self.authorize(ctx)?;
        let name = std::str::from_utf8(id)
            .ok()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ConnectError::invalid_argument("instance ref.id is required"))?;
        self.runner
            .app(name)
            .ok_or_else(|| ConnectError::not_found("app not configured on this runner"))
    }
}
#[allow(refining_impl_trait)]
impl InstanceService for Instances {
    async fn list(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceListRequest>,
    ) -> ServiceResult<InstanceListResponse> {
        self.authorize(&ctx)?;
        let req = req.to_owned_message();
        if !(0..=1000).contains(&req.size) {
            return Err(ConnectError::invalid_argument(
                "size must be between 0 and 1000",
            ));
        }
        let size = if req.size == 0 {
            100
        } else {
            req.size as usize
        };
        let mut items: Vec<_> = self
            .runner
            .list()
            .into_iter()
            .filter(|item| {
                req.filters.is_empty()
                    || req
                        .filters
                        .iter()
                        .any(|filter| filter.name.is_empty() || filter.name == item.name)
            })
            .collect();
        if req.reverse {
            items.reverse();
        }
        if !req.after.is_empty() {
            let offset = items
                .iter()
                .position(|item| item.name == req.after)
                .ok_or_else(|| {
                    ConnectError::invalid_argument("unknown instance cursor for these filters")
                })?;
            items.drain(..=offset);
        }
        let next = if items.len() > size {
            items[size - 1].name.clone()
        } else {
            String::new()
        };
        items.truncate(size);
        Response::ok(InstanceListResponse {
            items,
            next,
            ..Default::default()
        })
    }
    async fn get(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceGetRequest>,
    ) -> ServiceResult<Instance> {
        Response::ok(self.app(&ctx, &req.to_owned_message().r#ref.id)?.snapshot())
    }
    async fn start(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStartRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&ctx, &req.to_owned_message().r#ref.id)?;
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
        req: ServiceRequest<'_, InstanceStopRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&ctx, &req.to_owned_message().r#ref.id)?;
        app.stop(false, false)
            .await
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        Response::ok(app.snapshot())
    }
}
