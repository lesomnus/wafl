use crate::{
    config::INSTANCE_DOMAIN,
    proto::wafl::*,
    runner::{Execution, Runner},
    server::{self, Server},
    services::wafl::{InstanceService, InstanceServiceLayer, InstanceServiceOverlay},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn router(runner: Runner) -> connectrpc::Router {
    // Registration and layer composition use the normal entity contracts.
    // The runner supplies storage; this process does not select the DB Sink.
    let stack = server::Unimplemented.layer(Instances(runner));
    server::mount_if(connectrpc::Router::new(), &stack, |service| {
        service == "wafl.InstanceService"
    })
}

#[derive(Clone)]
struct Instances(Runner);
impl<N: Server> server::Layer<N> for Instances {
    fn wafl_instance(&self, next: &N) -> impl InstanceService {
        InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
}
impl Instances {
    fn app(&self, reference: &InstanceRef) -> Result<Arc<Execution>, ConnectError> {
        self.0
            .resolve(reference)?
            .and_then(|id| self.0.instance(&id))
            .ok_or_else(|| ConnectError::not_found("instance not found"))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    id: Vec<u8>,
    reverse: bool,
}

#[allow(refining_impl_trait)]
impl<N: InstanceService> InstanceServiceLayer<N> for Instances {
    async fn add(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceAddRequest>,
    ) -> ServiceResult<Instance> {
        Response::ok(self.0.add(req.to_owned_message()).await?)
    }
    async fn erase(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceRef>,
    ) -> ServiceResult<InstanceEraseResponse> {
        let req = req.to_owned_message();
        let erased = match self.0.resolve(&req)? {
            Some(id) => self.0.erase(&id).await?,
            None => false,
        };
        Response::ok(InstanceEraseResponse {
            erased: Some(erased),
            ..Default::default()
        })
    }
    async fn get(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceGetRequest>,
    ) -> ServiceResult<Instance> {
        let req = req.to_owned_message();
        let mut value = self.app(&req.r#ref)?.snapshot();
        if let Some(select) = req.select.as_option() {
            if let Some(related) = select.tenant.as_option()
                && (related.all.unwrap_or(false)
                    || related.alias.unwrap_or(false)
                    || related.name.unwrap_or(false)
                    || related.desc.unwrap_or(false)
                    || related.labels.unwrap_or(false)
                    || related.date_updated.unwrap_or(false)
                    || related.date_created.unwrap_or(false))
            {
                return Err(ConnectError::unimplemented(
                    "runner holds tenant references, not tenant metadata",
                ));
            }
            if !select.all.unwrap_or(false) {
                if !select.alias.unwrap_or(false) {
                    value.alias.clear();
                }
                if select.tenant.as_option().is_none() {
                    value.tenant = Default::default();
                }
                if !select.name.unwrap_or(false) {
                    value.name.clear();
                }
                if !select.phase.unwrap_or(false) {
                    value.phase = Default::default();
                }
                if !select.generation.unwrap_or(false) {
                    value.generation = 0;
                }
                if !select.active.unwrap_or(false) {
                    value.active = 0;
                }
                if !select.error.unwrap_or(false) {
                    value.error.clear();
                }
                if !select.manifest.unwrap_or(false) {
                    value.manifest = Default::default();
                }
            }
        }
        Response::ok(value)
    }
    async fn list(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceListRequest>,
    ) -> ServiceResult<InstanceListResponse> {
        let req = req.to_owned_message();
        if req.filters.len() > 64 {
            return Err(ConnectError::invalid_argument(
                "at most 64 filters are allowed",
            ));
        }
        let filters = req
            .filters
            .iter()
            .map(|filter| self.0.resolve(&filter.r#ref))
            .collect::<Result<Vec<_>, _>>()?;
        let size = if req.size <= 0 {
            100
        } else {
            req.size.min(1000) as usize
        };
        let mut items: Vec<_> = self
            .0
            .list()
            .into_iter()
            .filter(|item| filters.is_empty() || filters.iter().flatten().any(|id| item.id == *id))
            .collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        if req.reverse {
            items.reverse();
        }
        if !req.after.is_empty() {
            let invalid = || {
                ConnectError::invalid_argument("invalid instance cursor for this order or filter")
            };
            if req.after.len() > 512 {
                return Err(invalid());
            }
            let cursor: Cursor =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&req.after).map_err(|_| invalid())?)
                    .map_err(|_| invalid())?;
            if cursor.reverse != req.reverse || !paydirt::has_domain(&cursor.id, INSTANCE_DOMAIN) {
                return Err(invalid());
            }
            items.retain(|item| {
                if req.reverse {
                    item.id < cursor.id
                } else {
                    item.id > cursor.id
                }
            });
        }
        let next = if items.len() > size {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&Cursor {
                    id: items[size - 1].id.clone(),
                    reverse: req.reverse,
                })
                .map_err(|_| ConnectError::internal("cursor encoding failed"))?,
            )
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
    async fn start(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceStartRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&req.to_owned_message().r#ref)?;
        let lease = self
            .0
            .acquire(&app)
            .await
            .map_err(|e| ConnectError::unavailable(e.to_string()))?;
        drop(lease);
        Response::ok(app.snapshot())
    }
    async fn stop(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceStopRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&req.to_owned_message().r#ref)?;
        app.stop(false, false)
            .await
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        Response::ok(app.snapshot())
    }

    async fn tail(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, InstanceTailRequest>,
    ) -> ServiceResult<connectrpc::ServiceStream<Log>> {
        let req = req.to_owned_message();
        Response::ok(
            self.app(&req.r#ref)?
                .tail(req.cursor, req.limit, req.follow)?,
        )
    }
}
