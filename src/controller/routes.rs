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
            let mut middlewares = BTreeMap::new();
            let date = now();
            for app in apps {
                let name = format!("wafl-{}", paydirt::format_id(&app.id).unwrap());
                let ready: Vec<_> = placements.iter().filter(|p| {
                    p.app.id == app.id && p.revision == app.revision && p.phase == PlacementPhase::Ready
                        && date.saturating_sub(p.date_seen) <= config.fresh_ms && p.date_seen <= date
                        && p.execution.phase == Phase::Running && !p.execution.draining && p.execution.direct
                }).take(scaling::desired(&app) as usize).collect();
                routers.insert(name.clone(), json!({"rule":format!("Host(`{}`)",app.domain),"entryPoints":[config.entrypoint],"service":name}));
                if managed(&app) {
                    let t=config.traffic.as_ref().ok_or_else(||paydirt_store::Fault::precondition("traffic configuration missing"))?;
                    if ready.is_empty() {
                        let servers=if app.scaling.as_option().is_some() {
                            let url=if app.manifest.protocol=="h2c" {t.activate.replacen("http://","h2c://",1)} else {t.activate.clone()};
                            vec![json!({"url":url})]
                        } else {Vec::new()};
                        services.insert(name,json!({"loadBalancer":{"servers":servers,"passHostHeader":true}}));
                    } else {
                        let mut weighted=Vec::new();
                        for p in ready {
                            let leaf=format!("wafl-{}",paydirt::format_id(&p.id).unwrap());
                            middlewares.insert(leaf.clone(),json!({"plugin":{"wafl":{"control":t.control,"activate":t.activate,"placement":encode(&p.id),"intervalMs":t.interval_ms}}}));
                            services.insert(leaf.clone(),json!({"loadBalancer":{"servers":[{"url":format!("{}://{}",p.execution.endpoint.protocol,p.execution.endpoint.address)}],"passHostHeader":true},"middlewares":[leaf]}));
                            weighted.push(json!({"name":leaf,"weight":1}));
                        }
                        services.insert(name,json!({"weighted":{"services":weighted}}));
                    }
                } else {
                    let servers:Vec<_>=ready.iter().map(|p|json!({"url":format!("{}://{}",p.execution.endpoint.protocol,p.execution.endpoint.address)})).collect();
                    services.insert(name, json!({"loadBalancer":{"servers":servers,"passHostHeader":true}}));
                }
            }
            if routers.is_empty() {return Ok(json!({}));}
            let mut output=json!({"http":{"routers":routers,"services":services}});
            if !middlewares.is_empty() {output["http"]["middlewares"]=json!(middlewares);}
            Ok(output)
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
