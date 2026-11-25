use super::controller::*;
use super::*;
use serde_json::{Value, json};
use wafl_runner::proto::wafl::{PlacementPhase, Traffic};

fn port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
async fn observations(endpoint: &str) -> Vec<Traffic> {
    let (status, body) = rpc(endpoint, "TrafficService", "List", json!({"size":1000})).await;
    assert_eq!(status, 200, "{body}");
    serde_json::from_value(body.get("items").cloned().unwrap_or_else(|| json!([]))).unwrap()
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires PostgreSQL and Traefik; set WAFL_TEST_CONTROLLER_DATABASE_URL and WAFL_TEST_TRAEFIK"]
async fn yaml_first_request_load_scaling_idle_zero_and_cold_restart() {
    let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").unwrap();
    let traefik = std::env::var("WAFL_TEST_TRAEFIK").unwrap();
    let (sql, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
        .await
        .unwrap();
    let database_task = tokio::spawn(driver);
    let schema = format!("autoscale_{}", uuid::Uuid::new_v4().simple());
    sql.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut database = reqwest::Url::parse(&database).unwrap();
    database
        .query_pairs_mut()
        .append_pair("options", &format!("-c search_path={schema}"));
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let first = port();
    assert!(first < 65504);
    let options = json!({"direct":{"listen":"127.0.0.1","advertise":"127.0.0.1","directory":root.path().join("ports"),"first":first,"last":first+31},"capacity":{"running":2}});
    let mut runner1 = Server::options(&cache, false, Some(options.clone())).await;
    let mut runner2 = Server::options(&cache, false, Some(options)).await;
    let control = port();
    let activation = port();
    let entry = port();
    let config = json!({"database":database.as_str().replace('+',"%20"),"runners":[runner1.endpoint,runner2.endpoint],"listen":format!("127.0.0.1:{control}"),
        "interval_ms":100,"probe_ms":200,"fresh_ms":3000,"max_pending":2,"max_replicas":3,
        "traffic":{"control":format!("http://127.0.0.1:{control}"),"activate":format!("http://127.0.0.1:{activation}"),"listen":format!("127.0.0.1:{activation}"),"interval_ms":100,"fresh_ms":1000}});
    let mut c1 = Controller::start(root.path(), &config).await;
    let mut second = config.clone();
    second["listen"] = json!("127.0.0.1:0");
    second["traffic"]["listen"] = json!("127.0.0.1:0");
    let mut c2 = Controller::start(root.path(), &second).await;
    let assets = Assets::new(false).await;
    let yaml = root.path().join("app.yaml");
    std::fs::write(&yaml,format!("alias: autoscale\ndomain: same.example\nmanifest:\n  bundle:\n    url: {}\n  env:\n    GREETING: auto\nscaling:\n  min: 0\n  max: 3\n  target: 1\n  up_ms: 100\n  down_ms: 1000\n  idle_ms: 1500\n  wait_ms: 20000\n",assets.url)).unwrap();
    let created = app(&c1.endpoint, &["add", yaml.to_str().unwrap()]).await;
    assert_eq!(created.state.desired, 0);
    assert_eq!(created.manifest.port, 8080);
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(placements(&c1.endpoint).await.is_empty());
    assert_eq!(assets.count.load(Ordering::SeqCst), 0);
    let plugin = root
        .path()
        .join("plugins-local/src/github.com/lesomnus/wafl/traefik");
    std::fs::create_dir_all(&plugin).unwrap();
    for name in ["wafl.go", "go.mod", ".traefik.yml"] {
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("plugins/traefik")
                .join(name),
            plugin.join(name),
        )
        .unwrap();
    }
    let mut gateway = Command::new(traefik)
        .current_dir(root.path())
        .args([
            format!("--entrypoints.web.address=127.0.0.1:{entry}"),
            format!("--providers.http.endpoint={}/traefik", c1.endpoint),
            "--providers.http.pollinterval=100ms".into(),
            "--providers.providersthrottleduration=100ms".into(),
            "--experimental.localplugins.wafl.modulename=github.com/lesomnus/wafl/traefik".into(),
            "--log.level=ERROR".into(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(gateway.stdout.take().unwrap()).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("autoscale traefik: {line}");
        }
    });
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            assert!(
                gateway.try_wait().unwrap().is_none(),
                "Traefik exited during startup"
            );
            if tokio::net::TcpStream::connect(("127.0.0.1", entry))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Traefik did not start");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let address = format!("http://127.0.0.1:{entry}");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    // One request, no retries. It must survive 0 -> provisioning -> response.
    let response = client
        .get(&address)
        .header("host", "same.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "auto:1");
    until(&c1.endpoint, |p| {
        p.iter().any(|p| p.phase == PlacementPhase::Ready)
    })
    .await;
    // Wait for Traefik to install the direct endpoint and register its gate.
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let response = client
                .get(format!("{address}/headers"))
                .header("host", "same.example")
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
            if observations(&c1.endpoint)
                .await
                .iter()
                .any(|t| !t.activation && t.started > 0)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let mut requests = tokio::task::JoinSet::new();
    let loaded = Arc::new(std::sync::atomic::AtomicBool::new(false));
    for _ in 0..24 {
        let client = client.clone();
        let address = address.clone();
        let loaded = loaded.clone();
        requests.spawn(async move {
            while !loaded.load(Ordering::SeqCst) {
                let response = client
                    .get(format!("{address}/slow"))
                    .header("host", "same.example")
                    .send()
                    .await
                    .unwrap();
                assert_eq!(response.status(), 200);
                assert!(response.text().await.unwrap().starts_with("auto:"));
            }
        });
    }
    let peak = until(&c1.endpoint, |p| {
        p.iter()
            .filter(|p| p.phase == PlacementPhase::Ready)
            .count()
            == 3
    })
    .await;
    assert!(
        peak.iter()
            .any(|p| p.runner == format!("{}/", runner1.endpoint))
    );
    assert!(
        peak.iter()
            .any(|p| p.runner == format!("{}/", runner2.endpoint))
    );
    loaded.store(true, Ordering::SeqCst);
    while let Some(result) = requests.join_next().await {
        result.unwrap();
    }
    until(&c1.endpoint, |p| {
        !p.is_empty() && p.iter().all(|p| p.phase == PlacementPhase::Released)
    })
    .await;
    let observed = app(&c2.endpoint, &["get", "@autoscale"]).await;
    assert_eq!(observed.state.desired, 0);
    assert_eq!(observed.state.active, 0);
    for endpoint in [&runner1.endpoint, &runner2.endpoint] {
        let (status, row) = rpc(
            endpoint,
            "RunnerService",
            "Get",
            json!({"ref":{"alias":"self"}}),
        )
        .await;
        assert_eq!(status, 200);
        assert_eq!(
            row["usage"]
                .get("instances")
                .and_then(Value::as_str)
                .unwrap_or("0"),
            "0"
        );
    }
    // A stale direct configuration must also fall back before sending bytes.
    let response = client
        .get(&address)
        .header("host", "same.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "auto:1");
    assert_eq!(
        assets.count.load(Ordering::SeqCst),
        1,
        "shared bundle cache must survive scale to zero"
    );
    let stopped = app(&c1.endpoint, &["patch", "@autoscale", "--fixed"]).await;
    assert!(stopped.scaling.as_option().is_none());
    until(&c1.endpoint, |p| {
        p.iter().all(|p| p.phase == PlacementPhase::Released)
    })
    .await;
    success(cli(&c1.endpoint, &["app", "erase", "@autoscale"]).await);
    grpc_streams(root.path(), &c1.endpoint, entry).await;
    gateway.kill().await.unwrap();
    gateway.wait().await.unwrap();
    c1.stop().await;
    c2.stop().await;
    runner1.stop().await;
    runner2.stop().await;
    sql.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    database_task.abort();
}

async fn grpc_streams(root: &Path, controller: &str, entry: u16) {
    let dir = PathBuf::from(std::env::var_os("WAFL_TEST_GUEST_DIR").unwrap());
    let wasm = std::fs::read(dir.join("grpc.wasm")).unwrap();
    let mut archive = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let mut header = tar::Header::new_gnu();
    header.set_size(wasm.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
        .append_data(&mut header, "main.wasm", wasm.as_slice())
        .unwrap();
    let bundle = root.join("grpc.tar.gz");
    std::fs::write(&bundle, archive.into_inner().unwrap().finish().unwrap()).unwrap();
    let yaml = root.join("grpc.yaml");
    std::fs::write(&yaml,format!("alias: grpc\ndomain: grpc.example\nmanifest:\n  protocol: h2c\n  bundle:\n    url: {}\nscaling:\n  min: 0\n  max: 1\n  down_ms: 1000\n  idle_ms: 1000\n",reqwest::Url::from_file_path(bundle).unwrap())).unwrap();
    let created = app(controller, &["add", yaml.to_str().unwrap()]).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let (mut client, driver) = h2::client::handshake(
        tokio::net::TcpStream::connect(("127.0.0.1", entry))
            .await
            .unwrap(),
    )
    .await
    .unwrap();
    let driver = tokio::spawn(driver);
    let data = bytes::Bytes::from_static(b"\x00\x00\x00\x00\x05\x0a\x03abc");
    for cold in [true, false] {
        eprintln!("gRPC stream cold={cold}");
        let request = http::Request::builder()
            .method("POST")
            .uri("http://grpc.example/echo.Echo/Listen")
            .header("content-type", "application/grpc")
            .header("grpc-timeout", "120S")
            .body(())
            .unwrap();
        let (reply, mut upload) = client.send_request(request, false).unwrap();
        upload.send_data(data.clone(), false).unwrap();
        let reply = tokio::time::timeout(Duration::from_secs(100), reply)
            .await
            .unwrap_or_else(|_| panic!("gRPC response headers timed out, cold={cold}"))
            .unwrap();
        assert_eq!(reply.status(), 200);
        let mut body = reply.into_body();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), body.data())
                .await
                .unwrap()
                .unwrap()
                .unwrap(),
            data
        );
        body.flow_control().release_capacity(data.len()).unwrap();
        if cold {
            // Full duplex during the very first activation, held longer than
            // idle+down windows. The still-open stream must keep its instance.
            tokio::time::sleep(Duration::from_millis(2300)).await;
            assert!(
                placements(controller)
                    .await
                    .iter()
                    .any(|p| p.app.id == created.id && p.phase == PlacementPhase::Ready)
            );
        } else {
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    if observations(controller)
                        .await
                        .iter()
                        .any(|t| t.app.id == created.id && !t.activation && t.active == 1)
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await
            .expect("direct gRPC stream was not observed");
            app(controller, &["patch", "@grpc", "--fixed"]).await;
            let retiring = until(controller, |p| {
                p.iter()
                    .any(|p| p.app.id == created.id && p.phase == PlacementPhase::Retiring)
            })
            .await;
            assert!(
                retiring
                    .iter()
                    .filter(|p| p.app.id == created.id)
                    .all(|p| !p.withdrawn)
            );
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        // A stream already accepted by a retiring endpoint continues normally.
        upload.send_data(data.clone(), true).unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), body.data())
                .await
                .unwrap()
                .unwrap()
                .unwrap(),
            data
        );
        body.flow_control().release_capacity(data.len()).unwrap();
        assert!(body.data().await.is_none());
        assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "0");
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    driver.abort();
    until(controller, |p| {
        p.iter()
            .filter(|p| p.app.id == created.id)
            .all(|p| p.phase == PlacementPhase::Released)
    })
    .await;
    success(cli(controller, &["app", "erase", "@grpc"]).await);
}
