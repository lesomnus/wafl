use super::*;
use crate::{
    server::{self, Server},
    services::wafl::*,
};
use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use paydirt_store::{Fault, Transaction};

pub fn router(controller: Control) -> connectrpc::Router {
    let stack = server::Unimplemented.layer(controller);
    server::mount_if(connectrpc::Router::new(), &stack, |name| {
        matches!(
            name,
            "wafl.AppService"
                | "wafl.PlacementService"
                | "wafl.ControllerService"
                | "wafl.TenantService"
                | "wafl.TrafficService"
        )
    })
}
impl<N: Server> server::Layer<N> for Control {
    fn wafl_traffic(&self, next: &N) -> impl TrafficService {
        TrafficServiceOverlay {
            next: next.wafl_traffic(),
            layer: self.clone(),
        }
    }
    fn wafl_app(&self, next: &N) -> impl AppService {
        AppServiceOverlay {
            next: next.wafl_app(),
            layer: self.clone(),
        }
    }
    fn wafl_placement(&self, next: &N) -> impl PlacementService {
        PlacementServiceOverlay {
            next: next.wafl_placement(),
            layer: self.clone(),
        }
    }
    fn wafl_controller(&self, next: &N) -> impl ControllerService {
        ControllerServiceOverlay {
            next: next.wafl_controller(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl TenantService {
        TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
}
fn value<T: serde::Serialize>(input: T) -> Value {
    serde_json::to_value(input).expect("protobuf JSON encoding")
}
fn manifest(tenant: &[u8], input: Manifest) -> Result<Manifest, ConnectError> {
    if input.idle_ms != 0 || input.max_active != 0 {
        return Err(ConnectError::invalid_argument(
            "controller uses direct mode; idle_ms and max_active must be zero",
        ));
    }
    let mut normalized = crate::config::Spec::parse(InstanceAddRequest {
        tenant: Some(TenantRef {
            key: Some(tenant_ref::Key::Id(tenant.to_vec())),
            ..Default::default()
        })
        .into(),
        manifest: Some(input).into(),
        ..Default::default()
    })?
    .manifest;
    normalized.idle_ms = 0;
    normalized.max_active = 0;
    Ok(normalized)
}
fn validate(app: &App, max: u32) -> Result<Manifest, ConnectError> {
    let name = &app.domain;
    if name.is_empty()
        || name.len() > 253
        || !name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        })
    {
        return Err(ConnectError::invalid_argument(
            "domain must be an exact lowercase ASCII DNS hostname",
        ));
    }
    if app.replicas > max {
        return Err(ConnectError::invalid_argument(
            "replicas exceeds controller max_replicas",
        ));
    }
    if app.name.len() > 256 || app.name.contains('\0') {
        return Err(ConnectError::invalid_argument(
            "name exceeds 256 bytes or contains NUL",
        ));
    }
    manifest(
        &app.tenant.id,
        app.manifest
            .as_option()
            .cloned()
            .ok_or_else(|| ConnectError::invalid_argument("manifest is required"))?,
    )
}
fn invalid(error: ConnectError) -> Fault {
    Fault::invalid(error.message.unwrap_or_default())
}
fn finish_patch(
    tx: &mut Transaction<'_>,
    old: App,
    mut updated: App,
    config: &Config,
) -> paydirt_store::Result<App> {
    let normalized = validate(&updated, config.max_replicas).map_err(invalid)?;
    let policy = normalize_scaling(&updated, config).map_err(invalid)?;
    let changed =
        normalized != *old.manifest || policy.is_some() != old.scaling.as_option().is_some();
    let state = if changed {
        initial_state(policy.as_ref())
    } else {
        (*old.state).clone()
    };
    updated.revision = old
        .revision
        .checked_add(u64::from(changed))
        .ok_or_else(|| Fault::precondition("app revision exhausted"))?;
    decode(call(
        tx,
        "App",
        "Patch",
        json!({"ref":{"id":encode(&updated.id)},"manifest":normalized,"revision":updated.revision.to_string(),"scaling":policy,"state":state}),
    )?)
}
fn normalize_scaling(app: &App, config: &Config) -> Result<Option<Scaling>, ConnectError> {
    app.scaling
        .as_option()
        .map(|p| {
            if config.traffic.is_none() || app.replicas != 0 {
                return Err(ConnectError::invalid_argument(
                    "scaling requires controller traffic configuration and replicas=0",
                ));
            }
            scaling::normalize(p.clone(), config.max_replicas)
        })
        .transpose()
}
fn initial_state(policy: Option<&Scaling>) -> ScaleState {
    ScaleState {
        desired: policy.map_or(0, |p| p.min),
        date_activity: now(),
        date_sample: now(),
        date_change: now(),
        fresh: true,
        ..Default::default()
    }
}
macro_rules! read {
    ($name:ident,$request:ty,$response:ty,$entity:literal,$method:literal) => {
        async fn $name(
            &self,
            _: &N,
            _: RequestContext,
            req: ServiceRequest<'_, $request>,
        ) -> ServiceResult<$response> {
            let request = value(req.to_owned_message());
            Response::ok(
                self.db
                    .transaction(move |tx| decode(call(tx, $entity, $method, request)?))
                    .await?,
            )
        }
    };
}
#[allow(refining_impl_trait)]
impl<N: AppService> AppServiceLayer<N> for Control {
    read!(get, AppGetRequest, App, "App", "Get");
    read!(list, AppListRequest, AppListResponse, "App", "List");
    async fn add(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, AppAddRequest>,
    ) -> ServiceResult<App> {
        let req = req.to_owned_message();
        if req.revision != 0 || req.state.as_option().is_some() {
            return Err(ConnectError::invalid_argument(
                "revision and state are server-set",
            ));
        }
        let config = self.config.clone();
        let request = value(req);
        Response::ok(
            self.db
                .transaction(move |tx| {
                    let mut row = call(tx, "App", "Add", request)?;
                    row["revision"] = "1".into();
                    let app: App = decode(row)?;
                    let normalized = validate(&app, config.max_replicas).map_err(invalid)?;
                    let policy=normalize_scaling(&app,&config).map_err(invalid)?;
                    let state=initial_state(policy.as_ref());
                    decode(call(
                        tx,
                        "App",
                        "Patch",
                        json!({"ref":{"id":encode(&app.id)},"revision":"1","manifest":normalized,"scaling":policy,"state":state}),
                    )?)
                })
                .await?,
        )
    }
    async fn patch(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, AppPatchRequest>,
    ) -> ServiceResult<App> {
        let req = req.to_owned_message();
        if req.manifest_null.unwrap_or(false) {
            return Err(ConnectError::invalid_argument("manifest cannot be cleared"));
        }
        let config = self.config.clone();
        let mut request = value(&req);
        if request.get("state").is_some() || request.get("stateNull").is_some() {
            return Err(ConnectError::invalid_argument("state is server-set"));
        }
        request.as_object_mut().unwrap().remove("revision");
        Response::ok(
            self.db
                .transaction(move |tx| {
                    let old: App = decode(call(tx, "App", "Get", json!({"ref":request["ref"]}))?)?;
                    if req.revision.is_some_and(|r| r != old.revision) {
                        return Err(Fault::aborted("app revision changed"));
                    }
                    let updated = decode(call(tx, "App", "Patch", request)?)?;
                    finish_patch(tx, old, updated, &config)
                })
                .await?,
        )
    }
    async fn apply(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, AppApplyRequest>,
    ) -> ServiceResult<App> {
        let request = value(req.to_owned_message());
        let config = self.config.clone();
        Response::ok(
            self.db
                .transaction(move |tx| {
                    let old: App = decode(call(tx, "App", "Get", json!({"ref":request["ref"]}))?)?;
                    let updated = decode(call(tx, "App", "Apply", request)?)?;
                    finish_patch(tx, old, updated, &config)
                })
                .await?,
        )
    }
    async fn erase(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, AppRef>,
    ) -> ServiceResult<AppEraseResponse> {
        let reference = value(req.to_owned_message());
        Response::ok(self.db.transaction(move |tx| {
            let page = call(tx, "App", "List", json!({"filters":[{"ref":reference}]}))?;
            let Some(found) = page["items"].as_array().and_then(|a| a.first()) else { return Ok(AppEraseResponse { erased: Some(false), ..Default::default() }); };
            let app: App = decode(found.clone())?;
            let placements = all::<Placement>(tx, "Placement")?.into_iter().filter(|p| p.app.id == app.id).collect::<Vec<_>>();
            if app.scaling.as_option().is_some() || app.replicas != 0 || placements.iter().any(|p| p.phase != PlacementPhase::Released) { return Err(Fault::precondition("disable scaling, set replicas=0 and release every placement before Erase")); }
            let traffic=all::<Traffic>(tx,"Traffic")?.into_iter().filter(|t|t.app.id==app.id).collect::<Vec<_>>();
            if traffic.iter().any(|t|!t.closed || t.active!=0) { return Err(Fault::precondition("traffic observers must close before Erase")); }
            for t in traffic { call(tx,"Traffic","Erase",json!({"id":encode(&t.id)}))?; }
            for p in placements { call(tx, "Placement", "Erase", json!({"id":encode(&p.id)}))?; }
            decode(call(tx, "App", "Erase", reference)?)
        }).await?)
    }
}
#[allow(refining_impl_trait)]
impl<N: PlacementService> PlacementServiceLayer<N> for Control {
    read!(get, PlacementGetRequest, Placement, "Placement", "Get");
    read!(
        list,
        PlacementListRequest,
        PlacementListResponse,
        "Placement",
        "List"
    );
    async fn release(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, PlacementReleaseRequest>,
    ) -> ServiceResult<Placement> {
        let req = req.to_owned_message();
        if !req.withdrawn {
            return Err(ConnectError::invalid_argument(
                "withdrawn=true must confirm all gateway routes, pools and streams are drained",
            ));
        }
        Response::ok(
            self.db
                .transaction(move |tx| {
                    let reference = value(&req.r#ref);
                    let p: Placement =
                        decode(call(tx, "Placement", "Get", json!({"ref":reference}))?)?;
                    if p.phase == PlacementPhase::Released {
                        return Ok(p);
                    }
                    if p.phase != PlacementPhase::Retiring {
                        return Err(Fault::precondition(
                            "placement must be retiring before acknowledging withdrawal",
                        ));
                    }
                    decode(call(
                        tx,
                        "Placement",
                        "Patch",
                        json!({"ref":reference,"withdrawn":true}),
                    )?)
                })
                .await?,
        )
    }
}
#[allow(refining_impl_trait)]
impl<N: ControllerService> ControllerServiceLayer<N> for Control {
    read!(get, ControllerGetRequest, Controller, "Controller", "Get");
}
#[allow(refining_impl_trait)]
impl<N: TenantService> TenantServiceLayer<N> for Control {
    read!(add, TenantAddRequest, Tenant, "Tenant", "Add");
    read!(get, TenantGetRequest, Tenant, "Tenant", "Get");
    read!(
        list,
        TenantListRequest,
        TenantListResponse,
        "Tenant",
        "List"
    );
}
