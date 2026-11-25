use super::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use wafl_runner::proto::wafl::{App, Placement, PlacementPhase};

pub(super) struct Controller {
    child: Child,
    pub(super) endpoint: String,
}
impl Controller {
    pub(super) async fn start(root: &Path, config: &Value) -> Self {
        let path = root.join(format!("controller-{}.json", uuid::Uuid::new_v4()));
        std::fs::write(&path, serde_json::to_vec(config).unwrap()).unwrap();
        let mut child = command()
            .args(["serve", "controller", "--config"])
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
        let address = tokio::time::timeout(Duration::from_secs(45), async {
            while let Some(line) = lines.next_line().await.unwrap() {
                if let Some(address) = line.strip_prefix("wafl-controller control=") {
                    return address.to_owned();
                }
                eprintln!("controller startup: {line}");
            }
            panic!("controller exited during startup");
        })
        .await
        .unwrap();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("controller: {line}");
            }
        });
        Self {
            child,
            endpoint: format!("http://{address}"),
        }
    }
    pub(super) async fn stop(&mut self) {
        self.child.kill().await.unwrap();
        self.child.wait().await.unwrap();
    }
}
pub(super) async fn rpc(
    endpoint: &str,
    service: &str,
    method: &str,
    request: Value,
) -> (u16, Value) {
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
        .post(format!("{endpoint}/wafl.{service}/{method}"))
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1")
        .body(request.to_string())
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    (
        status,
        serde_json::from_slice(&response.bytes().await.unwrap()).unwrap(),
    )
}
pub(super) async fn app(endpoint: &str, args: &[&str]) -> App {
    let mut all = vec!["--json", "app"];
    all.extend_from_slice(args);
    serde_json::from_slice(&success(cli(endpoint, &all).await)).unwrap()
}
pub(super) async fn placements(endpoint: &str) -> Vec<Placement> {
    let (status, body) = rpc(endpoint, "PlacementService", "List", json!({"size":1000})).await;
    assert_eq!(status, 200, "{body}");
    serde_json::from_value(body.get("items").cloned().unwrap_or_else(|| json!([]))).unwrap()
}
pub(super) async fn until(
    endpoint: &str,
    predicate: impl Fn(&[Placement]) -> bool,
) -> Vec<Placement> {
    tokio::time::timeout(Duration::from_secs(120), async {
        let mut previous = String::new();
        loop {
            let current = placements(endpoint).await;
            if predicate(&current) {
                return current;
            }
            let status = format!(
                "{:?}",
                current
                    .iter()
                    .map(|p| (&p.phase, &p.error))
                    .collect::<Vec<_>>()
            );
            if current.iter().any(|p| !p.error.is_empty()) && status != previous {
                eprintln!("placements: {status}");
            }
            previous = status;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    })
    .await
    .expect("placement reconciliation timeout")
}
async fn route(address: &str, domain: &str, prefix: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            if let Ok(response) = client.get(address).header("host", domain).send().await
                && response.status().is_success()
                && response.text().await.unwrap().starts_with(prefix)
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .expect("Traefik route did not become ready");
}
async fn restart(server: &Server, cache: &Path, options: &Value) -> Server {
    let control = server.endpoint.trim_start_matches("http://");
    let mut options = options.clone();
    options["listen"] = json!(server.ingress);
    options["control"] = json!(control);
    let path = cache.join("restart.json");
    std::fs::write(&path, options.to_string()).unwrap();
    let mut child = command()
        .args(["serve", "runner"])
        .arg(cache)
        .arg("--config")
        .arg(path)
        .args([
            "--control",
            control,
            "--listen",
            &server.ingress.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    tokio::time::timeout(Duration::from_secs(30), async {
        while let Some(line) = lines.next_line().await.unwrap() {
            if line == format!("wafl-runner ingress=direct control={control}") {
                return;
            }
            eprintln!("restarting runner: {line}");
        }
        panic!("runner restart failed");
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("runner: {line}");
        }
    });
    Server {
        child,
        endpoint: server.endpoint.clone(),
        ingress: server.ingress,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires PostgreSQL and Traefik; set WAFL_TEST_CONTROLLER_DATABASE_URL and WAFL_TEST_TRAEFIK"]
async fn postgres_replicas_reconcile_real_runners_and_traefik_routes_http_and_grpc() {
    let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").expect("PostgreSQL test URL");
    let traefik = std::env::var("WAFL_TEST_TRAEFIK").expect("Traefik executable");
    let (sql, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
        .await
        .unwrap();
    let database_task = tokio::spawn(driver);
    let schema = format!("controller_{}", uuid::Uuid::new_v4().simple());
    sql.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut database = reqwest::Url::parse(&database).unwrap();
    database
        .query_pairs_mut()
        .append_pair("options", &format!("-c search_path={schema}"));
    // libpq treats '+' literally in URL query values; use percent-encoded space.
    let database = database.as_str().replace('+', "%20");
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let first = probe.local_addr().unwrap().port();
    assert!(first < 65504);
    drop(probe);
    let options = json!({"direct":{"listen":"127.0.0.1","advertise":"127.0.0.1","directory":root.path().join("ports"),"first":first,"last":first+31},"capacity":{"running":3}});
    let mut a = Server::options(&cache, false, Some(options.clone())).await;
    let mut b = Server::options(&cache, false, Some(options.clone())).await;
    let config = json!({"database":database,"runners":[a.endpoint,b.endpoint],"listen":"127.0.0.1:0","interval_ms":200,"probe_ms":500,"fresh_ms":4000,"max_pending":2,"max_replicas":4});
    let (mut c1, mut c2) = tokio::join!(
        Controller::start(root.path(), &config),
        Controller::start(root.path(), &config)
    );
    let assets = Assets::new(false).await;
    let alpha = app(
        &c1.endpoint,
        &[
            "add",
            &assets.url,
            "--alias",
            "alpha",
            "--domain",
            "same.example",
            "--replicas",
            "2",
            "--env",
            "GREETING=alpha",
        ],
    )
    .await;
    let beta = app(
        &c2.endpoint,
        &[
            "add",
            &assets.url,
            "--alias",
            "beta",
            "--domain",
            "beta.example",
            "--env",
            "GREETING=beta",
        ],
    )
    .await;
    let initial = until(&c1.endpoint, |p| {
        p.iter()
            .filter(|p| p.phase == PlacementPhase::Ready)
            .count()
            == 3
    })
    .await;
    assert_eq!(
        initial.len(),
        3,
        "replicas must not duplicate placement reservations"
    );
    assert!(
        initial
            .iter()
            .any(|p| p.runner.trim_end_matches('/') == a.endpoint)
    );
    assert!(
        initial
            .iter()
            .any(|p| p.runner.trim_end_matches('/') == b.endpoint)
    );
    assert!(
        app(&c2.endpoint, &["get", "@alpha"])
            .await
            .manifest
            .bundle
            .digest
            .starts_with("sha256:")
    );
    let snapshot = |endpoint: String| async move {
        serde_json::from_str::<Value>(
            &reqwest::get(format!("{endpoint}/traefik"))
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap()
    };
    assert_eq!(
        snapshot(c1.endpoint.clone()).await,
        snapshot(c2.endpoint.clone()).await
    );
    assert_eq!(
        rpc(
            &c1.endpoint,
            "AppService",
            "Patch",
            json!({"ref":{"alias":"alpha"},"replicas":99})
        )
        .await
        .0,
        400
    );
    assert_eq!(app(&c1.endpoint, &["get", "@alpha"]).await.replicas, 2);
    assert!(
        !cli(&c1.endpoint, &["app", "erase", "@alpha"])
            .await
            .status
            .success()
    );
    assert_eq!(
        rpc(
            &c1.endpoint,
            "AppService",
            "Patch",
            json!({"ref":{"alias":"alpha"},"revision":"999","replicas":1})
        )
        .await
        .1["code"],
        "aborted"
    );
    let p = &initial[0];
    assert_eq!(
        rpc(
            &c1.endpoint,
            "PlacementService",
            "Release",
            json!({"ref":{"id":STANDARD.encode(&p.id)},"withdrawn":true})
        )
        .await
        .1["code"],
        "failed_precondition"
    );
    // A zero-replica app supports validated, versioned desired-state updates.
    let idle = app(
        &c1.endpoint,
        &[
            "add",
            &assets.url,
            "--alias",
            "idle",
            "--domain",
            "idle.example",
            "--replicas",
            "0",
        ],
    )
    .await;
    let mut changed = (*idle.manifest).clone();
    changed.env.insert("GREETING".into(), "changed".into());
    let (status, body) = rpc(
        &c1.endpoint,
        "AppService",
        "Patch",
        json!({"ref":{"alias":"idle"},"revision":"1","manifest":changed}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["revision"], "2");
    // Apply uses the same validation/transaction as Patch, including rollback.
    let (status, body) = rpc(
        &c1.endpoint,
        "AppService",
        "Apply",
        json!({
            "ref":{"alias":"idle"},"patch":{"delta":{"entries":[{
                "targets":{"selectors":[{"key":{"field":{"name":"replicas"}}}]},
                "assign":{"value":{"u32":99}}
            }]}}
        }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(app(&c1.endpoint, &["get", "@idle"]).await.replicas, 0);

    let gateway_socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let gateway_address = gateway_socket.local_addr().unwrap();
    drop(gateway_socket);
    let mut gateway = Command::new(traefik)
        .args([
            format!("--entrypoints.web.address={gateway_address}"),
            format!("--providers.http.endpoint={}/traefik", c2.endpoint),
            "--providers.http.pollinterval=100ms".into(),
            "--providers.http.polltimeout=2s".into(),
            "--providers.providersthrottleduration=100ms".into(),
            "--log.level=ERROR".into(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let url = format!("http://{gateway_address}/");
    route(&url, "same.example", "alpha:").await;
    route(&url, "beta.example", "beta:").await;
    route(&format!("{url}headers"), "same.example", "preserved").await;
    assert_eq!(
        reqwest::Client::new()
            .get(&url)
            .header("host", "idle.example")
            .send()
            .await
            .unwrap()
            .status(),
        503
    );
    success(cli(&c1.endpoint, &["app", "erase", "@idle"]).await);
    assert_eq!(
        reqwest::Client::new()
            .get(&url)
            .header("host", "unknown.example")
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    // Controllers can restart while the database and existing executions remain.
    let ids: BTreeSet<_> = initial.iter().map(|p| p.instance.clone()).collect();
    c1.stop().await;
    c1 = Controller::start(root.path(), &config).await;
    let current = placements(&c1.endpoint).await;
    assert_eq!(ids, current.iter().map(|p| p.instance.clone()).collect());
    route(&url, "same.example", "alpha:").await;
    // Kill a real runner. A new placement on the surviving runner is routed;
    // the old endpoint stays reserved until explicit withdrawal acknowledgement.
    let dead_endpoint = a.endpoint.clone();
    let dead_ids: BTreeSet<_> = current
        .iter()
        .filter(|p| p.runner.trim_end_matches('/') == dead_endpoint)
        .map(|p| p.id.clone())
        .collect();
    a.stop().await;
    let recovered = until(&c2.endpoint, |p| {
        p.iter()
            .filter(|p| dead_ids.contains(&p.id))
            .all(|p| p.phase == PlacementPhase::Retiring)
            && p.iter()
                .filter(|p| p.phase == PlacementPhase::Ready)
                .count()
                >= 2
    })
    .await;
    assert!(
        recovered
            .iter()
            .filter(|p| dead_ids.contains(&p.id))
            .all(|p| !p.withdrawn)
    );
    a = restart(&a, &cache, &options).await;
    until(&c2.endpoint, |p| {
        p.iter()
            .filter(|p| p.phase == PlacementPhase::Ready)
            .count()
            == 3
    })
    .await;
    route(&url, "same.example", "alpha:").await;
    route(&url, "beta.example", "beta:").await;

    // An h2c WASI server is reached through Traefik with ordinary gRPC framing.
    let dir = PathBuf::from(std::env::var_os("WAFL_TEST_GUEST_DIR").unwrap());
    let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let wasm = std::fs::read(dir.join("grpc.wasm")).unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_size(wasm.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
        .append_data(&mut header, "main.wasm", wasm.as_slice())
        .unwrap();
    let path = root.path().join("grpc.tar.gz");
    std::fs::write(&path, archive.into_inner().unwrap().finish().unwrap()).unwrap();
    app(
        &c1.endpoint,
        &[
            "add",
            reqwest::Url::from_file_path(path).unwrap().as_str(),
            "--alias",
            "grpc",
            "--domain",
            "grpc.example",
            "--protocol",
            "h2c",
        ],
    )
    .await;
    until(&c2.endpoint, |p| {
        p.iter()
            .filter(|p| p.phase == PlacementPhase::Ready)
            .count()
            == 4
    })
    .await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    let (mut h2, connection) = h2::client::handshake(
        tokio::net::TcpStream::connect(gateway_address)
            .await
            .unwrap(),
    )
    .await
    .unwrap();
    let connection = tokio::spawn(connection);
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.example/echo.Echo/Listen")
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let (reply, mut output) = h2.send_request(request, false).unwrap();
    let data = bytes::Bytes::from_static(b"\x00\x00\x00\x00\x05\x0a\x03abc");
    output.send_data(data.clone(), true).unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(10), reply)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reply.status(), 200);
    let mut body = reply.into_body();
    assert_eq!(body.data().await.unwrap().unwrap(), data);
    body.flow_control().release_capacity(data.len()).unwrap();
    assert!(body.data().await.is_none());
    assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "0");
    connection.abort();

    for alias in ["@alpha", "@beta", "@grpc"] {
        app(&c1.endpoint, &["patch", alias, "--replicas", "0"]).await;
    }
    let retired = until(&c1.endpoint, |p| {
        p.iter().all(|p| p.phase == PlacementPhase::Retiring)
    })
    .await;
    // Stop the only gateway and wait for process exit: no route or connection
    // can survive. Configuration receipt alone is deliberately not used as ACK.
    Command::new("kill")
        .args(["-TERM", &gateway.id().unwrap().to_string()])
        .status()
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(20), gateway.wait())
        .await
        .unwrap()
        .unwrap();
    for p in &retired {
        let id = paydirt::format_id(&p.id).unwrap();
        assert!(
            !cli(&c1.endpoint, &["placement", "release", &id])
                .await
                .status
                .success()
        );
        success(cli(&c1.endpoint, &["placement", "release", &id, "--withdrawn"]).await);
    }
    until(&c2.endpoint, |p| {
        p.iter().all(|p| p.phase == PlacementPhase::Released)
    })
    .await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert!(
        placements(&c2.endpoint)
            .await
            .iter()
            .all(|p| p.error.is_empty()),
        "late release failures cannot overwrite terminal success"
    );
    for alias in ["@alpha", "@beta", "@grpc"] {
        success(cli(&c1.endpoint, &["app", "erase", alias]).await);
    }
    assert!(placements(&c2.endpoint).await.is_empty());
    assert_eq!(snapshot(c2.endpoint.clone()).await, json!({}));
    assert_ne!(alpha.id, beta.id);
    // A bad revision remains inspectable, without creating a new failed
    // execution every reconciliation tick. Changing the manifest permits repair.
    let failed = app(
        &c1.endpoint,
        &[
            "add",
            &assets.url,
            "--alias",
            "failed",
            "--domain",
            "failed.example",
        ],
    )
    .await;
    let running = until(&c1.endpoint, |p| {
        p.len() == 1 && p[0].phase == PlacementPhase::Ready
    })
    .await;
    let _ = reqwest::get(format!(
        "http://{}/exit",
        running[0].execution.endpoint.address
    ))
    .await;
    until(&c1.endpoint, |p| {
        p.len() == 1
            && p[0].phase == PlacementPhase::Retiring
            && p[0].execution.phase == Phase::Failed
    })
    .await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_eq!(placements(&c2.endpoint).await.len(), 1);
    let mut repaired = (*failed.manifest).clone();
    repaired.env.insert("GREETING".into(), "repaired".into());
    let (status, body) = rpc(
        &c1.endpoint,
        "AppService",
        "Patch",
        json!({"ref":{"alias":"failed"},"manifest":repaired}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    until(&c2.endpoint, |p| {
        p.len() == 2
            && p.iter()
                .any(|p| p.revision == 2 && p.phase == PlacementPhase::Ready)
    })
    .await;
    app(&c1.endpoint, &["patch", "@failed", "--replicas", "0"]).await;
    let retired = until(&c1.endpoint, |p| {
        p.iter().all(|p| p.phase == PlacementPhase::Retiring)
    })
    .await;
    for p in retired {
        success(
            cli(
                &c1.endpoint,
                &[
                    "placement",
                    "release",
                    &paydirt::format_id(&p.id).unwrap(),
                    "--withdrawn",
                ],
            )
            .await,
        );
    }
    until(&c1.endpoint, |p| {
        p.iter().all(|p| p.phase == PlacementPhase::Released)
    })
    .await;
    success(cli(&c1.endpoint, &["app", "erase", "@failed"]).await);
    // Startup failure rolls Add back on the runner. Its durable intent keeps
    // the same ID and applies a shared retry deadline instead of spinning.
    app(
        &c1.endpoint,
        &[
            "add",
            &assets.url,
            "--alias",
            "startup",
            "--domain",
            "startup.example",
            "--env",
            "MODE=exit",
        ],
    )
    .await;
    let backoff = until(&c1.endpoint, |p| {
        p.len() == 1 && p[0].failures >= 2 && p[0].date_retry > p[0].date_seen
    })
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let current = placements(&c2.endpoint).await;
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].instance, backoff[0].instance);
    assert_eq!(current[0].date_retry, backoff[0].date_retry);
    c1.stop().await;
    c2.stop().await;
    a.stop().await;
    b.stop().await;
    sql.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    drop(sql);
    database_task.abort();
}
