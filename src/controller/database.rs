use super::*;
use paydirt_store::{Call, Fault, Scope, Store, Transaction};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

pub const TENANT: &str = "00000000-0000-8000-8001-000000000001";
const COORDINATOR: &str = "00000000-0000-8000-800c-000000000001";
const MODEL: &str = include_str!("../../generated/storage/model.json");
const POLICIES: &str = include_str!("../../generated/storage/policies.json");

#[derive(Clone)]
pub struct Database(Arc<Mutex<Store>>, String);
pub fn encode(id: &[u8]) -> String {
    STANDARD.encode(id)
}
pub fn decode<T: DeserializeOwned>(value: Value) -> paydirt_store::Result<T> {
    serde_json::from_value(value).map_err(|_| Fault::internal("invalid stored controller record"))
}
pub fn call(
    tx: &mut Transaction<'_>,
    entity: &str,
    method: &str,
    request: Value,
) -> paydirt_store::Result<Value> {
    tx.call(Call {
        entity: format!("wafl.{entity}"),
        method: method.into(),
        request,
    })
}
pub fn all<T: DeserializeOwned>(
    tx: &mut Transaction<'_>,
    entity: &str,
) -> paydirt_store::Result<Vec<T>> {
    let mut after = Value::String(String::new());
    let mut result = Vec::new();
    loop {
        let page = call(tx, entity, "List", json!({"size":1000,"after":after}))?;
        for item in page["items"].as_array().into_iter().flatten() {
            result.push(decode(item.clone())?);
        }
        after = page["next"].clone();
        if after.as_str().is_none_or(str::is_empty) {
            return Ok(result);
        }
    }
}
pub fn fault(error: Fault) -> ConnectError {
    match error.code.as_str() {
        "invalid_argument" => ConnectError::invalid_argument(error.message),
        "not_found" => ConnectError::not_found(error.message),
        "already_exists" => ConnectError::already_exists(error.message),
        "failed_precondition" => ConnectError::failed_precondition(error.message),
        "aborted" => ConnectError::aborted(error.message),
        "resource_exhausted" => ConnectError::resource_exhausted(error.message),
        _ => ConnectError::internal("controller database operation failed"),
    }
}
impl Database {
    pub async fn open(config: &Config, configure: bool) -> anyhow::Result<Self> {
        let url = config.database.clone();
        let signature = config.signature();
        tokio::task::spawn_blocking(move || {
            let mut store = Store::open(&url, MODEL, POLICIES)?.with_outbox(false);
            // Bootstrapping can race across replicas. Each seed transaction either
            // commits or rolls back; retry then verifies the shared configuration.
            for attempt in 0..3 {
                match store.seed(vec![
                    ("wafl.Tenant".into(), json!({"id":encode(&paydirt::parse_id(TENANT).unwrap()),"alias":"local","name":"Local development"})),
                    ("wafl.Controller".into(), json!({"id":encode(&paydirt::parse_id(COORDINATOR).unwrap()),"alias":"self","configuration":signature})),
                ]) {
                    Ok(()) => break,
                    Err(e) if e.code == "already_exists" && attempt < 2 => continue,
                    Err(e) => return Err(e),
                }
            }
            let current = store.operator(Call { entity:"wafl.Controller".into(), method:"Get".into(), request:json!({"ref":{"alias":"self"}}) })?;
            if current["configuration"] != signature {
                if !configure { return Err(Fault::precondition("controller configuration changed; stop old replicas and use --configure to adopt it")); }
                store.operator(Call { entity:"wafl.Controller".into(), method:"Patch".into(), request:json!({"ref":{"alias":"self"},"configuration":signature}) })?;
            }
            Ok(Self(Arc::new(Mutex::new(store)), signature))
        }).await?.map_err(|e: Fault| anyhow::anyhow!(fault(e)))
    }
    pub async fn transaction<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Transaction<'_>) -> paydirt_store::Result<T> + Send + 'static,
    ) -> Result<T, ConnectError> {
        let db = self.0.clone();
        let signature = self.1.clone();
        tokio::task::spawn_blocking(move || {
            let mut store = db
                .lock()
                .map_err(|_| Fault::internal("database worker stopped"))?;
            store.transaction(&Scope::Internal, |tx| {
                // A no-op Patch acquires PostgreSQL FOR UPDATE. All decisions
                // are short transactions; no network calls hold this lock.
                let current=call(tx, "Controller", "Patch", json!({"ref":{"alias":"self"}}))?;
                if current["configuration"] != signature { return Err(Fault::precondition("controller configuration was replaced; restart with the current configuration")); }
                operation(tx)
            })
        })
        .await
        .map_err(|_| ConnectError::internal("database worker failed"))?
        .map_err(fault)
    }
}
