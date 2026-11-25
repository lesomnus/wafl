use super::*;
use paydirt_server::axum::{self, response::IntoResponse, routing::get};
use std::collections::BTreeMap;

impl Control {
    pub async fn configuration(&self) -> Result<Value, ConnectError> {
        let config = self.config.clone();
        self.db.transaction(move |tx| {
            let apps = all::<App>(tx, "App")?;
            let mut placements = all::<Placement>(tx, "Placement")?;
            placements.sort_by(|a,b| a.id.cmp(&b.id));
            let mut routers = BTreeMap::new();
            let mut services = BTreeMap::new();
            let date = now();
            for app in apps {
                let name = format!("wafl-{}", paydirt::format_id(&app.id).unwrap());
                let servers: Vec<_> = placements.iter().filter(|p| {
                    p.app.id == app.id && p.revision == app.revision && p.phase == PlacementPhase::Ready
                        && date.saturating_sub(p.date_seen) <= config.fresh_ms && p.date_seen <= date
                        && p.execution.phase == Phase::Running && !p.execution.draining && p.execution.direct
                }).take(app.replicas as usize).map(|p| json!({"url":format!("{}://{}",p.execution.endpoint.protocol,p.execution.endpoint.address)})).collect();
                routers.insert(name.clone(), json!({"rule":format!("Host(`{}`)",app.domain),"entryPoints":[config.entrypoint],"service":name}));
                services.insert(name, json!({"loadBalancer":{"servers":servers,"passHostHeader":true}}));
            }
            Ok(json!({"http":{"routers":routers,"services":services}}))
        }).await
    }
}
pub fn router(controller: Control) -> axum::Router {
    axum::Router::new().route(
        "/traefik",
        get(move || {
            let controller = controller.clone();
            async move {
                match controller.configuration().await {
                    Ok(value) => (
                        [
                            ("content-type", "application/json"),
                            ("cache-control", "no-store"),
                        ],
                        value.to_string(),
                    )
                        .into_response(),
                    Err(_) => (
                        http::StatusCode::SERVICE_UNAVAILABLE,
                        "controller database unavailable",
                    )
                        .into_response(),
                }
            }
        }),
    )
}
