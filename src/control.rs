use crate::{
    config::{INSTANCE_DOMAIN, TENANT_DOMAIN},
    proto::wafl::*,
    runner::{App, Runner},
    server::{self, Server},
    services::wafl::{InstanceService, InstanceServiceLayer, InstanceServiceOverlay},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use connectrpc::{
    ConnectError, Encodable, RequestContext, Response, ServiceRequest, ServiceResult,
};
use paydirt::tenant::TenantTokens;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub fn tokens(json: &str) -> anyhow::Result<TenantTokens> {
    let configured: HashMap<String, String> = serde_json::from_str(json)?;
    anyhow::ensure!(
        !configured.is_empty(),
        "at least one control token is required"
    );
    let principals = configured
        .into_iter()
        .map(|(token, id)| {
            let id = paydirt::parse_id(&id)
                .ok_or_else(|| anyhow::anyhow!("invalid Tenant ID in WAFL_TOKENS"))?;
            Ok((token, id.to_vec()))
        })
        .collect::<anyhow::Result<_>>()?;
    TenantTokens::new(principals, TENANT_DOMAIN).map_err(anyhow::Error::msg)
}

pub fn router(runner: Runner, tokens: TenantTokens) -> connectrpc::Router {
    // Registration and layer composition use the normal entity contracts.
    // The runner supplies storage; this process does not select the DB Sink.
    let stack = server::Unimplemented
        .layer(Instances(runner))
        .layer(Authentication(Arc::new(tokens)));
    server::mount_if(connectrpc::Router::new(), &stack, |service| {
        service == "wafl.InstanceService"
    })
}

#[derive(Clone)]
struct Authentication(Arc<TenantTokens>);
#[derive(Clone, Copy)]
struct TenantScope([u8; 16]);

impl Authentication {
    fn resolve(&self, mut ctx: RequestContext) -> Result<RequestContext, ConnectError> {
        let mut values = ctx.headers().get_all("authorization").iter();
        let tenant = values
            .next()
            .and_then(|value| value.to_str().ok())
            .and_then(|value| self.0.resolve(value))
            .filter(|_| values.next().is_none())
            .ok_or_else(|| {
                ConnectError::unauthenticated("a provisioned tenant token is required")
            })?;
        ctx.extensions_mut()
            .insert(TenantScope(tenant.try_into().expect("validated Tenant ID")));
        Ok(ctx)
    }
}
impl<N: Server> server::Layer<N> for Authentication {
    fn wafl_instance(&self, next: &N) -> impl InstanceService {
        InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
}
#[allow(refining_impl_trait)]
impl<N: InstanceService> InstanceServiceLayer<N> for Authentication {
    async fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceGetRequest>,
    ) -> ServiceResult<impl Encodable<Instance> + Send + use<'a, N>> {
        next.get(self.resolve(ctx)?, req).await
    }
    async fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceListRequest>,
    ) -> ServiceResult<impl Encodable<InstanceListResponse> + Send + use<'a, N>> {
        next.list(self.resolve(ctx)?, req).await
    }
    async fn start<'a>(
        &'a self,
        next: &'a N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStartRequest>,
    ) -> ServiceResult<impl Encodable<Instance> + Send + use<'a, N>> {
        next.start(self.resolve(ctx)?, req).await
    }
    async fn stop<'a>(
        &'a self,
        next: &'a N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStopRequest>,
    ) -> ServiceResult<impl Encodable<Instance> + Send + use<'a, N>> {
        next.stop(self.resolve(ctx)?, req).await
    }
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
fn tenant(ctx: &RequestContext) -> Result<[u8; 16], ConnectError> {
    ctx.extensions()
        .get::<TenantScope>()
        .map(|scope| scope.0)
        .ok_or_else(|| ConnectError::unauthenticated("tenant scope required"))
}
fn id(reference: &InstanceRef) -> Result<&[u8], ConnectError> {
    match &reference.key {
        Some(instance_ref::Key::Id(id)) if paydirt::has_domain(id, INSTANCE_DOMAIN) => Ok(id),
        _ => Err(ConnectError::invalid_argument(
            "ref.id must be a domain-7 Payday ID",
        )),
    }
}
impl Instances {
    fn app(&self, ctx: &RequestContext, reference: &InstanceRef) -> Result<Arc<App>, ConnectError> {
        let scope = tenant(ctx)?;
        self.0
            .instance(id(reference)?)
            .filter(|app| app.spec.tenant.0 == scope)
            .ok_or_else(|| ConnectError::not_found("instance not found"))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    tenant: [u8; 16],
    id: Vec<u8>,
    reverse: bool,
}

#[allow(refining_impl_trait)]
impl<N: InstanceService> InstanceServiceLayer<N> for Instances {
    async fn get(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceGetRequest>,
    ) -> ServiceResult<Instance> {
        let req = req.to_owned_message();
        let mut value = self.app(&ctx, &req.r#ref)?.snapshot();
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
                if !select.domain.unwrap_or(false) {
                    value.domain.clear();
                }
                if !select.port.unwrap_or(false) {
                    value.port = 0;
                }
            }
        }
        Response::ok(value)
    }
    async fn list(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceListRequest>,
    ) -> ServiceResult<InstanceListResponse> {
        let scope = tenant(&ctx)?;
        let req = req.to_owned_message();
        if req.filters.len() > 64 {
            return Err(ConnectError::invalid_argument(
                "at most 64 filters are allowed",
            ));
        }
        let filters = req
            .filters
            .iter()
            .map(|filter| id(&filter.r#ref))
            .collect::<Result<Vec<_>, _>>()?;
        let size = if req.size <= 0 {
            100
        } else {
            req.size.min(1000) as usize
        };
        // Narrow by tenant before filtering, pagination or deciding whether a
        // next page exists. Other tenants cannot affect this caller's pages.
        let mut items: Vec<_> = self
            .0
            .list()
            .into_iter()
            .filter(|item| item.tenant.id == scope)
            .filter(|item| filters.is_empty() || filters.iter().any(|id| item.id == *id))
            .collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        if req.reverse {
            items.reverse();
        }
        if !req.after.is_empty() {
            let invalid = || {
                ConnectError::invalid_argument(
                    "invalid instance cursor for this tenant, order or filter",
                )
            };
            if req.after.len() > 512 {
                return Err(invalid());
            }
            let cursor: Cursor =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&req.after).map_err(|_| invalid())?)
                    .map_err(|_| invalid())?;
            if cursor.tenant != scope || cursor.reverse != req.reverse {
                return Err(invalid());
            }
            let offset = items
                .iter()
                .position(|item| item.id == cursor.id)
                .ok_or_else(invalid)?;
            items.drain(..=offset);
        }
        let next = if items.len() > size {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&Cursor {
                    tenant: scope,
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
        let app = self.app(&ctx, &req.to_owned_message().r#ref)?;
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
        ctx: RequestContext,
        req: ServiceRequest<'_, InstanceStopRequest>,
    ) -> ServiceResult<Instance> {
        let app = self.app(&ctx, &req.to_owned_message().r#ref)?;
        app.stop(false, false)
            .await
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        Response::ok(app.snapshot())
    }
}
