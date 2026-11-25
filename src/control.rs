use crate::{
    config::{INSTANCE_DOMAIN, RESERVATION_DOMAIN},
    proto::wafl::*,
    runner::{Condition, Execution, Runner},
    server::{self, Server},
    services::wafl::{
        InstanceService, InstanceServiceLayer, InstanceServiceOverlay, ReservationService,
        ReservationServiceLayer, ReservationServiceOverlay, RunnerService, RunnerServiceLayer,
        RunnerServiceOverlay,
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod reservations;

pub fn router(runner: Runner) -> connectrpc::Router {
    // Registration and layer composition use the normal entity contracts.
    // The runner supplies storage; this process does not select the DB Sink.
    let stack = server::Unimplemented.layer(Instances(runner));
    server::mount_if(connectrpc::Router::new(), &stack, |service| {
        matches!(
            service,
            "wafl.InstanceService" | "wafl.RunnerService" | "wafl.ReservationService"
        )
    })
}

#[derive(Clone)]
struct Instances(Runner);
impl<N: Server> server::Layer<N> for Instances {
    fn wafl_reservation(&self, next: &N) -> impl ReservationService {
        ReservationServiceOverlay {
            next: next.wafl_reservation(),
            layer: self.clone(),
        }
    }
    fn wafl_runner(&self, next: &N) -> impl RunnerService {
        RunnerServiceOverlay {
            next: next.wafl_runner(),
            layer: self.clone(),
        }
    }
    fn wafl_instance(&self, next: &N) -> impl InstanceService {
        InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
}
impl Instances {
    fn app(&self, reference: &InstanceRef) -> Result<Arc<Execution>, ConnectError> {
        let id = self
            .0
            .resolve(reference)?
            .ok_or_else(|| ConnectError::not_found("instance not found"))?;
        self.0.instance(&id).ok_or_else(|| {
            if self.0.snapshot(&id).is_some() {
                ConnectError::failed_precondition("instance is still being added")
            } else {
                ConnectError::not_found("instance not found")
            }
        })
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
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceAddRequest>,
    ) -> ServiceResult<Instance> {
        let condition = condition(&ctx)?;
        if condition.revision.is_some() {
            return Err(ConnectError::invalid_argument(
                "Add accepts session, not revision",
            ));
        }
        self.0.check_session(condition.session)?;
        Response::ok(self.0.add(req.to_owned_message()).await?)
    }
    async fn erase(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceRef>,
    ) -> ServiceResult<InstanceEraseResponse> {
        let req = req.to_owned_message();
        let condition = condition(&ctx)?;
        self.0.check_session(condition.session)?;
        let erased = match self.0.resolve(&req)? {
            Some(id) => self.0.erase_condition(&id, condition).await?,
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
        let mut value = self
            .0
            .resolve(&req.r#ref)?
            .and_then(|id| self.0.snapshot(&id))
            .ok_or_else(|| ConnectError::not_found("instance not found"))?;
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
                if !select.session.unwrap_or(false) {
                    value.session.clear();
                }
                if !select.revision.unwrap_or(false) {
                    value.revision = 0;
                }
                if !select.draining.unwrap_or(false) {
                    value.draining = false;
                }
                if !select.endpoint.unwrap_or(false) {
                    value.endpoint = Default::default();
                }
                if !select.direct.unwrap_or(false) {
                    value.direct = false;
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
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStartRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&req.to_owned_message().r#ref)?;
        let lease = self.0.start(&app, condition(&ctx)?).await?;
        drop(lease);
        Response::ok(app.snapshot())
    }
    async fn stop(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStopRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&req.to_owned_message().r#ref)?;
        app.stop_condition(false, false, condition(&ctx)?, false)
            .await?;
        Response::ok(app.snapshot())
    }

    async fn drain(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceDrainRequest>,
    ) -> ServiceResult<Instance> {
        let req = req.to_owned_message();
        let app = self.app(&req.r#ref)?;
        app.drain(req.timeout_ms, req.force, condition(&ctx)?)
            .await?;
        Response::ok(app.snapshot())
    }

    async fn release(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceReleaseRequest>,
    ) -> ServiceResult<Instance> {
        let req = req.to_owned_message();
        let app = self.app(&req.r#ref)?;
        app.release(req.generation, condition(&ctx)?).await?;
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

// Shared by all management mutations, including generated Add and Erase.
pub(crate) fn condition(ctx: &RequestContext) -> Result<Condition, ConnectError> {
    fn single<'a>(ctx: &'a RequestContext, name: &str) -> Result<Option<&'a str>, ConnectError> {
        let mut values = ctx.headers().get_all(name).iter();
        let value = values
            .next()
            .map(|v| {
                v.to_str()
                    .map_err(|_| ConnectError::invalid_argument("invalid command condition"))
            })
            .transpose()?;
        if values.next().is_some() {
            return Err(ConnectError::invalid_argument(
                "duplicate command condition",
            ));
        }
        Ok(value)
    }
    let session = single(ctx, "wafl-session")?
        .map(|value| {
            paydirt::parse_id(value)
                .filter(|id| paydirt::has_domain(id, 8))
                .ok_or_else(|| {
                    ConnectError::invalid_argument("wafl-session must be a domain-8 Runner UUID")
                })
        })
        .transpose()?;
    let revision = single(ctx, "wafl-revision")?
        .map(|value| {
            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ConnectError::invalid_argument(
                    "wafl-revision must be decimal uint64",
                ));
            }
            value
                .parse::<u64>()
                .map_err(|_| ConnectError::invalid_argument("wafl-revision must be decimal uint64"))
        })
        .transpose()?;
    if revision.is_some() && session.is_none() {
        return Err(ConnectError::invalid_argument(
            "wafl-revision requires wafl-session",
        ));
    }
    Ok(Condition { session, revision })
}
impl Instances {
    fn runner_match(&self, reference: &RunnerRef) -> Result<bool, ConnectError> {
        match &reference.key {
            Some(runner_ref::Key::Id(id)) if paydirt::has_domain(id, 8) => {
                Ok(id.as_slice() == self.0.session())
            }
            Some(runner_ref::Key::Alias(alias)) if crate::config::valid_alias(alias) => {
                Ok(alias == "self")
            }
            _ => Err(ConnectError::invalid_argument(
                "runner ref must contain a domain-8 ID or alias",
            )),
        }
    }
}
#[allow(refining_impl_trait)]
impl<N: RunnerService> RunnerServiceLayer<N> for Instances {
    async fn get(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, RunnerGetRequest>,
    ) -> ServiceResult<crate::proto::wafl::Runner> {
        let req = req.to_owned_message();
        if !self.runner_match(&req.r#ref)? {
            return Err(ConnectError::not_found("runner session not found"));
        }
        let mut value = self.0.status();
        if let Some(select) = req.select.as_option()
            && !select.all.unwrap_or(false)
        {
            if !select.alias.unwrap_or(false) {
                value.alias.clear();
            }
            if !select.version.unwrap_or(false) {
                value.version.clear();
            }
            if !select.runtime.unwrap_or(false) {
                value.runtime.clear();
            }
            if !select.abi.unwrap_or(false) {
                value.abi.clear();
            }
            if !select.ingress.unwrap_or(false) {
                value.ingress.clear();
            }
            if !select.control.unwrap_or(false) {
                value.control.clear();
            }
            if !select.accepting.unwrap_or(false) {
                value.accepting = false;
            }
            if !select.capacity.unwrap_or(false) {
                value.capacity = Default::default();
            }
            if !select.usage.unwrap_or(false) {
                value.usage = Default::default();
            }
            if !select.capabilities.unwrap_or(false) {
                value.capabilities.clear();
            }
        }
        Response::ok(value)
    }
    async fn list(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, RunnerListRequest>,
    ) -> ServiceResult<RunnerListResponse> {
        let req = req.to_owned_message();
        if req.filters.len() > 64 {
            return Err(ConnectError::invalid_argument(
                "at most 64 filters are allowed",
            ));
        }
        let filters = req
            .filters
            .iter()
            .map(|f| self.runner_match(&f.r#ref))
            .collect::<Result<Vec<_>, _>>()?;
        // There is at most one row, so this service never emits a next cursor.
        if !req.after.is_empty() {
            return Err(ConnectError::invalid_argument(
                "RunnerService has no continuation cursor",
            ));
        }
        let items = if filters.is_empty() || filters.contains(&true) {
            vec![self.0.status()]
        } else {
            Vec::new()
        };
        Response::ok(RunnerListResponse {
            items,
            ..Default::default()
        })
    }
}
