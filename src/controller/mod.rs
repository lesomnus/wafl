mod api;
pub mod config;
mod database;
mod reconcile;
mod routes;

use crate::proto::wafl::*;
use base64::{Engine, engine::general_purpose::STANDARD};
pub use config::Config;
use connectrpc::ConnectError;
use database::{Database, all, call, decode, encode};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Control {
    config: Arc<Config>,
    db: Database,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
impl Control {
    pub async fn open(mut config: Config) -> anyhow::Result<Self> {
        config.validate()?;
        for endpoint in &mut config.runners {
            *endpoint = reqwest::Url::parse(endpoint)?.to_string();
        }
        let db = Database::open(&config).await?;
        Ok(Self {
            config: Arc::new(config),
            db,
        })
    }
}
pub async fn serve(config: Config) -> anyhow::Result<()> {
    use paydirt::spin::{Lease, Placement as WorkerPlacement, Workers};
    let controller = Control::open(config).await?;
    let host = paydirt_server::Host::bind_with_routes(
        controller.config.listen,
        Default::default(),
        "wafl-controller",
        api::router(controller.clone()),
        routes::router(controller.clone()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    eprintln!("wafl-controller control={}", host.local_addr()?);
    let mut workers = Workers::default();
    workers
        .add("wafl.reconcile", WorkerPlacement::Everywhere, move |stop| {
            let controller = controller.clone();
            async move { controller.run(stop).await }
        })
        .map_err(anyhow::Error::msg)?;
    workers
        .run(
            Lease::Local,
            crate::host::shutdown(),
            move |stop| async move { host.serve(stop).await.map_err(|e| e.to_string()) },
        )
        .await
        .map_err(anyhow::Error::msg)
}
