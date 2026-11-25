use super::controller::*;
use super::*;
use serde_json::json;
use wafl_runner::proto::wafl::{App, Placement, PlacementPhase};

const ALPHA: &str = "synthetic-secret-alpha\n";
const BETA: &str = "synthetic-secret-beta";

async fn ready(controller: &Controller, app: &App) -> Placement {
    until(&controller.endpoint, |ps| {
        ps.iter().any(|p| {
            p.app.id == app.id && p.revision == app.revision && p.phase == PlacementPhase::Ready
        })
    })
    .await
    .into_iter()
    .find(|p| p.app.id == app.id && p.revision == app.revision && p.phase == PlacementPhase::Ready)
    .unwrap()
}
async fn greeting(p: &Placement) -> String {
    reqwest::get(format!("http://{}/", p.execution.endpoint.address))
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
fn no_values(value: &impl serde::Serialize) {
    let json = serde_json::to_string(value).unwrap();
    for secret in ["synthetic-secret-alpha", BETA] {
        assert!(
            !json.contains(secret),
            "secret leaked into control metadata"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires PostgreSQL; set WAFL_TEST_CONTROLLER_DATABASE_URL"]
async fn file_secrets_reach_guest_without_persisting_or_echoing_values() {
    let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").unwrap();
    let (sql, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
        .await
        .unwrap();
    let driver = tokio::spawn(driver);
    let schema = format!("secrets_{}", uuid::Uuid::new_v4().simple());
    sql.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut database = reqwest::Url::parse(&database).unwrap();
    database
        .query_pairs_mut()
        .append_pair("options", &format!("-c search_path={schema}"));
    let root = tempfile::tempdir().unwrap();
    let files = root.path().join("secret-files");
    let cache = root.path().join("cache");
    std::fs::create_dir(&files).unwrap();
    std::fs::create_dir(&cache).unwrap();
    std::fs::write(files.join("current"), ALPHA).unwrap();
    let first = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    assert!(first < 65504);
    let mut runner=Server::options(&cache,false,Some(json!({"direct":{"listen":"127.0.0.1","advertise":"127.0.0.1","first":first,"last":first+31,"directory":root.path().join("ports")}}))).await;
    let config = json!({"database":database.as_str().replace('+',"%20"),"runners":[runner.endpoint],"listen":"127.0.0.1:0","interval_ms":100,"probe_ms":500,"fresh_ms":4000,
        "secrets":{"files":{"00000000-0000-8000-8001-000000000001":files}}});
    let mut controller = Controller::start(root.path(), &config).await;
    let assets = Assets::new(false).await;
    let yaml = root.path().join("app.yaml");
    std::fs::write(&yaml,format!("alias: secret\ndomain: secret.example\nmanifest:\n  bundle: {{url: '{}'}}\nsecrets:\n  - env: GREETING\n    file: current\n",assets.url)).unwrap();
    let original = app(&controller.endpoint, &["add", yaml.to_str().unwrap()]).await;
    no_values(&original);
    let p = ready(&controller, &original).await;
    assert_eq!(greeting(&p).await, format!("{ALPHA}:1"));
    no_values(&p);
    assert!(p.execution.secrets.is_empty());
    assert!(!p.execution.manifest.env.contains_key("GREETING"));
    let (status,snapshot)=rpc(&runner.endpoint,"InstanceService","Get",json!({"ref":{"id":STANDARD.encode(&p.instance)},"select":{"secrets":true,"manifest":true}})).await;
    assert_eq!(status, 200);
    no_values(&snapshot);
    let (_, snapshot) = rpc(
        &runner.endpoint,
        "InstanceService",
        "List",
        json!({"select":{"secrets":true,"manifest":true}}),
    )
    .await;
    no_values(&snapshot);

    // Editing the source does not mutate a running instance or App revision.
    std::fs::write(files.join("current"), BETA).unwrap();
    assert_eq!(greeting(&p).await, format!("{ALPHA}:2"));
    assert_eq!(
        app(&controller.endpoint, &["get", "@secret"])
            .await
            .revision,
        original.revision
    );
    std::fs::write(files.join("next"), BETA).unwrap();
    let bindings = root.path().join("bindings.yaml");
    std::fs::write(&bindings, "- env: GREETING\n  file: next\n").unwrap();
    let changed = app(
        &controller.endpoint,
        &["patch", "@secret", "--secrets", bindings.to_str().unwrap()],
    )
    .await;
    assert_eq!(changed.revision, original.revision + 1);
    let second = ready(&controller, &changed).await;
    assert_eq!(greeting(&second).await, format!("{BETA}:1"));
    let _ = reqwest::get(format!(
        "http://{}/secret-error",
        second.execution.endpoint.address
    ))
    .await;
    let failed = until(&controller.endpoint, |ps| {
        ps.iter()
            .any(|p| p.id == second.id && p.execution.phase == Phase::Failed)
    })
    .await;
    no_values(&failed);
    assert!(
        failed
            .iter()
            .any(|p| p.id == second.id && p.error.contains("secret-bearing guest"))
    );
    let (_, snapshot) = rpc(
        &runner.endpoint,
        "InstanceService",
        "Get",
        json!({"ref":{"id":STANDARD.encode(&second.instance)}}),
    )
    .await;
    no_values(&snapshot);
    let (status, body) = rpc(
        &controller.endpoint,
        "PlacementService",
        "Release",
        json!({"ref":{"id":STANDARD.encode(&p.id)},"withdrawn":true}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    // Explicit [] clears a repeated field through generated AppService.Apply.
    std::fs::write(&bindings, "[]").unwrap();
    let cleared = app(
        &controller.endpoint,
        &["patch", "@secret", "--secrets", bindings.to_str().unwrap()],
    )
    .await;
    assert!(cleared.secrets.is_empty());
    assert_eq!(cleared.revision, changed.revision + 1);
    assert_eq!(greeting(&ready(&controller, &cleared).await).await, ":1");
    let (status, body) = rpc(
        &controller.endpoint,
        "PlacementService",
        "Release",
        json!({"ref":{"id":STANDARD.encode(&second.id)},"withdrawn":true}),
    )
    .await;
    assert_eq!(status, 200, "{body}");

    let request = json!({"alias":"invalid","tenant":{"alias":"local"},"domain":"invalid.example","manifest":{"port":8080,"bundle":{"url":assets.url}}});
    for bindings in [
        json!([{"env":"PORT","file":"current"}]),
        json!([{"env":"A","file":"../escape"}]),
        json!([{"env":"A","secretKeyRef":{"name":"x","key":"y"}}]),
        json!([{"env":"A","file":"current"},{"env":"A","file":"next"}]),
    ] {
        let mut bad = request.clone();
        bad["secrets"] = bindings;
        let (status, _) = rpc(&controller.endpoint, "AppService", "Add", bad).await;
        assert_eq!(status, 400);
    }
    let mut collision = request.clone();
    collision["secrets"] = json!([{"env":"A","file":"current"}]);
    collision["manifest"]["env"] = json!({"A":"public"});
    assert_eq!(
        rpc(&controller.endpoint, "AppService", "Add", collision)
            .await
            .0,
        400
    );
    // An unavailable source retains desired state and retries without starting
    // a partial guest; an application error must not persist its stderr.
    let mut failure = request;
    failure["alias"] = json!("failure");
    failure["secrets"] = json!([{"env":"GREETING","file":"later"}]);
    failure["manifest"]["env"] = json!({"MODE":"secret-error"});
    failure["replicas"] = json!(1);
    let (status, body) = rpc(&controller.endpoint, "AppService", "Add", failure).await;
    assert_eq!(status, 200, "{body}");
    let failure: App = serde_json::from_value(body).unwrap();
    let waiting = until(&controller.endpoint, |ps| {
        ps.iter()
            .any(|p| p.app.id == failure.id && !p.error.is_empty())
    })
    .await;
    assert!(
        waiting
            .iter()
            .filter(|p| p.app.id == failure.id)
            .all(|p| p.execution.id.is_empty())
    );
    std::fs::write(files.join("later"), BETA).unwrap();
    let failed = until(&controller.endpoint, |ps| {
        ps.iter()
            .any(|p| p.app.id == failure.id && p.error.contains("secret-bearing guest"))
    })
    .await;
    no_values(&failed);
    assert!(
        failed
            .iter()
            .any(|p| p.app.id == failure.id && p.error.contains("secret-bearing guest"))
    );
    let (_, snapshot) = rpc(
        &runner.endpoint,
        "InstanceService",
        "List",
        json!({"size":1000}),
    )
    .await;
    no_values(&snapshot);
    // Audit/trail tables, executions and resources must all remain value-free.
    let tables = sql
        .query(
            "SELECT tablename FROM pg_tables WHERE schemaname=$1",
            &[&schema],
        )
        .await
        .unwrap();
    for table in tables {
        let table: String = table.get(0);
        for row in sql
            .query(
                &format!("SELECT row_to_json(t)::text FROM {schema}.\"{table}\" t"),
                &[],
            )
            .await
            .unwrap()
        {
            let data: String = row.get(0);
            no_values(&data);
        }
    }
    controller.stop().await;
    runner.stop().await;
    sql.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    driver.abort();
}
