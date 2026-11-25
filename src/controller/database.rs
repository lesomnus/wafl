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
            // Match normal transactions' lock order: coordinator before tenant.
            // Reversing these deadlocks with a live placement's tenant FK check.
            for attempt in 0..3 {
                match store.seed(vec![
                    ("wafl.Controller".into(), json!({"id":encode(&paydirt::parse_id(COORDINATOR).unwrap()),"alias":"self","configuration":signature})),
                    ("wafl.Tenant".into(), json!({"id":encode(&paydirt::parse_id(TENANT).unwrap()),"alias":"local","name":"Local development"})),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires WAFL_TEST_CONTROLLER_DATABASE_URL"]
    async fn bootstrap_waits_for_coordinator_without_locking_the_tenant() {
        let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").unwrap();
        let (admin, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
            .await
            .unwrap();
        let driver = tokio::spawn(driver);
        let schema = format!("bootstrap_{}", uuid::Uuid::new_v4().simple());
        admin
            .batch_execute(&format!("CREATE SCHEMA {schema}"))
            .await
            .unwrap();
        let mut url = reqwest::Url::parse(&database).unwrap();
        url.query_pairs_mut()
            .append_pair("options", &format!("-c search_path={schema}"));
        let mut config: Config = serde_json::from_value(
            json!({"database":url.as_str().replace('+',"%20"),"runners":["http://127.0.0.1:9"]}),
        )
        .unwrap();
        let first = Database::open(&config, false).await.unwrap();
        let (mut live, connection) =
            tokio_postgres::connect(&config.database, tokio_postgres::NoTls)
                .await
                .unwrap();
        let connection = tokio::spawn(connection);
        let tx = live.transaction().await.unwrap();
        // A live controller has acquired the coordinator and is about to
        // insert a resource with a tenant FK, which needs KEY SHARE on Tenant.
        tx.query_one(
            "SELECT f1 FROM \"pd_wafl__Controller\" WHERE f4='self' FOR UPDATE",
            &[],
        )
        .await
        .unwrap();
        let name = format!("joining-{schema}");
        url.query_pairs_mut().append_pair("application_name", &name);
        config.database = url.as_str().replace('+', "%20");
        let joining = tokio::spawn(async move { Database::open(&config, false).await });
        // Observe the real server lock wait instead of guessing with a sleep.
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                let row=admin.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE application_name=$1 AND wait_event_type='Lock' AND query LIKE '%pd_wafl__Controller%FOR UPDATE%')",&[&name]).await.unwrap();
                if row.get::<_,bool>(0) {break;}
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }).await.unwrap();
        let tenant = tx
            .query_one(
                "SELECT f1 FROM \"pd_wafl__Tenant\" WHERE f4='local' FOR KEY SHARE NOWAIT",
                &[],
            )
            .await;
        tx.rollback().await.unwrap();
        let joined = tokio::time::timeout(Duration::from_secs(10), joining)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        drop(joined);
        drop(first);
        drop(live);
        connection.abort();
        admin
            .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
            .await
            .unwrap();
        driver.abort();
        assert!(
            tenant.is_ok(),
            "joining controller held Tenant while waiting for Controller: {tenant:?}"
        );
    }
}
