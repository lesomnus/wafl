use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper_util::rt::TokioIo;
use paydirt::spin::{Lease, Workers};
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::JoinHandle,
};
use wafl_runner::{
    config::Config,
    proto::wafl::*,
    proxy,
    runner::{Execution, Runner},
};

fn guest(name: &str) -> PathBuf {
    let base = std::env::var_os("WAFL_TEST_GUEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/wasm32-wasip2/debug")
        });
    let file = base.join(format!("{name}.wasm"));
    assert!(
        file.exists(),
        "missing {}: run bash scripts/test.sh",
        file.display()
    );
    file
}
struct TestApp {
    id: [u8; 16],
    tenant: [u8; 16],
    name: String,
    guest: PathBuf,
    manifest: Manifest,
}
impl std::ops::Deref for TestApp {
    type Target = Manifest;
    fn deref(&self) -> &Manifest {
        &self.manifest
    }
}
impl std::ops::DerefMut for TestApp {
    fn deref_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }
}
impl TestApp {
    fn request(&self, url: String) -> InstanceAddRequest {
        InstanceAddRequest {
            id: Some(self.id.to_vec()),
            name: self.name.clone(),
            tenant: Some(TenantRef {
                key: Some(tenant_ref::Key::Id(self.tenant.to_vec())),
                ..Default::default()
            })
            .into(),
            manifest: Some(Manifest {
                bundle: Some(Asset {
                    kind: Some(asset::Kind::Url(url)),
                    ..Default::default()
                })
                .into(),
                ..self.manifest.clone()
            })
            .into(),
            ..Default::default()
        }
    }
    fn bundle(&self) -> Vec<u8> {
        archive(&[
            ("main.wasm", std::fs::read(&self.guest).unwrap()),
            (
                "data/index.html",
                format!("<h1>{}</h1>", self.name).into_bytes(),
            ),
        ])
    }
}
fn archive(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    for (path, bytes) in files {
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(bytes.len() as u64);
        header.set_cksum();
        tar.append_data(&mut header, path, bytes.as_slice())
            .unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap()
}
struct Assets {
    base: String,
    task: JoinHandle<()>,
}
impl Assets {
    async fn new(files: BTreeMap<String, Vec<u8>>) -> Self {
        use paydirt_server::axum::{self, extract::Path, routing::get};
        let files = Arc::new(files);
        let router = axum::Router::new().route(
            "/{file}",
            get(move |Path(file): Path<String>| {
                let files = files.clone();
                async move {
                    match files.get(&file) {
                        Some(bytes) => (http::StatusCode::OK, bytes.clone()),
                        None => (http::StatusCode::NOT_FOUND, Vec::new()),
                    }
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self { base, task }
    }
    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base)
    }
}
impl Drop for Assets {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn app(name: &str, artifact: &str) -> TestApp {
    TestApp {
        id: paydirt::mint(7),
        tenant: fixed(1, 1),
        name: name.into(),
        guest: guest(artifact),
        manifest: Manifest {
            port: 8080,
            idle_ms: 100,
            env: [("GREETING".into(), name.into())].into_iter().collect(),
            ..Default::default()
        },
    }
}
fn fixed(domain: u8, number: u8) -> [u8; 16] {
    [
        0, 0, 0, 0, 0, 0, 0x80, 0, 0x80, domain, 0, 0, 0, 0, 0, number,
    ]
}
fn config() -> Config {
    Config {
        listen: "127.0.0.1:0".parse().unwrap(),
        control: "127.0.0.1:0".parse().unwrap(),
        directory: None,
        metrics: None,
        bundle: Default::default(),
        cache: Default::default(),
        capacity: Default::default(),
        direct: None,
    }
}
struct Harness {
    runner: Runner,
    address: SocketAddr,
    ids: BTreeMap<String, [u8; 16]>,
    _assets: Assets,
    _directory: tempfile::TempDir,
    stop: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}
impl Harness {
    async fn new(apps: Vec<TestApp>) -> Self {
        Self::limits(apps, Default::default()).await
    }
    async fn limits(apps: Vec<TestApp>, capacity: wafl_runner::config::Capacity) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let runner = Runner::new(&Config {
            directory: Some(directory.path().to_owned()),
            capacity,
            ..config()
        })
        .await
        .unwrap();
        let assets = Assets::new(
            apps.iter()
                .map(|app| (app.name.clone(), app.bundle()))
                .collect(),
        )
        .await;
        let mut ids = BTreeMap::new();
        for app in &apps {
            let instance = runner
                .add(app.request(assets.url(&app.name)))
                .await
                .unwrap();
            ids.insert(app.name.clone(), instance.id.try_into().unwrap());
        }
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, end) = oneshot::channel();
        let traffic = runner.clone();
        let task = tokio::spawn(async move {
            Workers::default()
                .run(
                    Lease::Local,
                    async {
                        let _ = end.await;
                    },
                    move |stop| async move {
                        proxy::serve(listener, traffic, stop)
                            .await
                            .map_err(|e| e.to_string())
                    },
                )
                .await
                .unwrap();
        });
        Self {
            runner,
            address,
            ids,
            _assets: assets,
            _directory: directory,
            stop: Some(stop),
            task,
        }
    }
    fn instance(&self, name: &str) -> Arc<Execution> {
        self.runner.instance(&self.ids[name]).unwrap()
    }
    async fn client(&self) -> Client {
        Client::new(self.address, self.ids.clone()).await
    }
    async fn finish(mut self) {
        let _ = self.stop.take().unwrap().send(());
        self.task.await.unwrap();
        self.runner.shutdown().await;
    }
}
struct Client {
    sender: hyper::client::conn::http1::SendRequest<Empty<Bytes>>,
    driver: JoinHandle<()>,
    ids: BTreeMap<String, [u8; 16]>,
}
impl Client {
    async fn new(address: SocketAddr, ids: BTreeMap<String, [u8; 16]>) -> Self {
        let (sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(
            TcpStream::connect(address).await.unwrap(),
        ))
        .await
        .unwrap();
        Self {
            sender,
            ids,
            driver: tokio::spawn(async {
                let _ = connection.await;
            }),
        }
    }
    async fn send(&mut self, name: &str, path: &str) -> hyper::Response<hyper::body::Incoming> {
        self.sender.ready().await.unwrap();
        let id = self.ids.get(name).copied().unwrap_or_else(|| fixed(7, 255));
        let request = hyper::Request::builder()
            .uri(path)
            .header("host", "same.example")
            .header(proxy::INSTANCE_HEADER, paydirt::format_id(&id).unwrap())
            .body(Empty::new())
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), self.sender.send_request(request))
            .await
            .unwrap()
            .unwrap()
    }
    async fn text(&mut self, name: &str, path: &str) -> String {
        let response = self.send(name, path).await;
        assert_eq!(response.status(), 200);
        String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap()
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.driver.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sockets_share_a_process_without_sharing_ports_or_state() {
    let h = Harness::new(vec![app("alpha", "socket"), app("beta", "socket")]).await;
    assert!(h.runner.list().iter().all(|i| i.phase == Phase::Running));
    let mut client = h.client().await;
    assert_eq!(client.text("alpha", "/").await, "alpha:1");
    assert_eq!(client.text("beta", "/").await, "beta:1");
    assert_eq!(client.text("alpha", "/").await, "alpha:2");
    assert_eq!(client.text("alpha", "/denied").await, "denied");
    let unknown = client.send("unknown", "/").await;
    assert_eq!(unknown.status(), 404);
    unknown.into_body().collect().await.unwrap();
    let alpha = h.instance("alpha");
    assert_eq!(alpha.snapshot().generation, 1);
    let response = client.send("alpha", "/slow").await;
    tokio::time::sleep(Duration::from_millis(180)).await;
    h.runner.reap().await;
    assert_eq!(alpha.snapshot().phase, Phase::Running);
    assert_eq!(alpha.snapshot().active, 1);
    assert!(alpha.stop(false, false).await.is_err());
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "alpha:4"
    );
    tokio::time::sleep(Duration::from_millis(150)).await;
    h.runner.reap().await;
    assert_eq!(alpha.snapshot().phase, Phase::Stopped);
    // The client keep-alive socket remains open across scale-to-zero.
    assert_eq!(client.text("alpha", "/").await, "alpha:1");
    assert_eq!(alpha.snapshot().generation, 2);
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn simultaneous_cold_requests_and_cancelled_starters_use_one_instance() {
    let mut spec = app("cold", "socket");
    spec.env.insert("MODE".into(), "delay".into());
    let h = Harness::new(vec![spec]).await;
    let app = h.instance("cold");
    app.stop(false, false).await.unwrap();
    let starter = {
        let runner = h.runner.clone();
        let app = app.clone();
        tokio::spawn(async move { runner.acquire(&app).await })
    };
    tokio::time::sleep(Duration::from_millis(20)).await;
    starter.abort();
    let _ = starter.await;
    let mut requests = tokio::task::JoinSet::new();
    for _ in 0..16 {
        let address = h.address;
        let ids = h.ids.clone();
        requests.spawn(async move { Client::new(address, ids).await.text("cold", "/").await });
    }
    let mut responses = std::collections::BTreeSet::new();
    while let Some(result) = requests.join_next().await {
        responses.insert(result.unwrap());
    }
    assert_eq!(responses.len(), 16);
    assert_eq!(app.snapshot().generation, 2);
    assert_eq!(app.snapshot().active, 0);
    let stopper = {
        let app = app.clone();
        tokio::spawn(async move { app.stop(false, false).await })
    };
    tokio::task::yield_now().await;
    stopper.abort();
    let _ = stopper.await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    // Stop may have completed, or cancellation may have preceded execution;
    // either way the app must not be left in Stopping.
    assert_ne!(app.snapshot().phase, Phase::Stopping);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn startup_failures_and_cpu_loops_do_not_block_other_apps() {
    let mut good = app("good", "socket");
    good.max_active = 1;
    let h = Harness::new(vec![good]).await;
    for mode in ["spin", "exit", "memory", "close"] {
        let mut bad = app(mode, "socket");
        bad.env.insert("MODE".into(), mode.into());
        bad.startup_ms = 120;
        if mode == "memory" {
            bad.memory_bytes = 2 * 1024 * 1024;
        }
        let files = Assets::new([(mode.into(), bad.bundle())].into()).await;
        let result = h.runner.add(bad.request(files.url(mode))).await;
        if let Ok(instance) = result {
            assert_eq!(mode, "close");
            tokio::time::sleep(Duration::from_millis(100)).await;
            assert_eq!(
                h.runner.instance(&instance.id).unwrap().snapshot().phase,
                Phase::Failed
            );
            h.runner.erase(&instance.id).await.unwrap();
        }
        assert!(h.runner.instance(&bad.id).is_none());
    }
    let good = h.instance("good");
    let lease = h.runner.acquire(&good).await.unwrap();
    assert!(h.runner.acquire(&good).await.is_err());
    drop(lease);
    assert_eq!(h.client().await.text("good", "/").await, "good:1");
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn management_contract_without_authentication_and_lifecycle() {
    use buffa::Message;
    use http_body_util::Full;
    use serde_json::json;
    use tower::ServiceExt;
    let mut apps = vec![];
    for (name, number, owner) in [
        ("alpha", 1, 1),
        ("beta", 2, 2),
        ("control", 3, 1),
        ("omega", 4, 2),
        ("zulu", 5, 1),
    ] {
        let mut value = app(name, "socket");
        value.id = fixed(7, number);
        value.tenant = fixed(1, owner);
        apps.push(value);
    }
    let h = Harness::new(apps).await;
    let runner = h.runner.clone();
    let service = connectrpc::ConnectRpcService::new(wafl_runner::control::router(runner.clone()));
    let call = |method: &str, body: serde_json::Value| {
        let service = service.clone();
        let request = http::Request::builder()
            .method("POST")
            .uri(format!("/wafl.InstanceService/{method}"))
            .header("content-type", "application/json")
            .header("connect-protocol-version", "1")
            .body(Full::new(Bytes::from(serde_json::to_vec(&body).unwrap())))
            .unwrap();
        async move {
            let response = service.oneshot(request).await.unwrap();
            let status = response.status();
            let body: serde_json::Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            (status, body)
        }
    };
    let reference = |number| json!({"id":STANDARD.encode(fixed(7,number))});
    for input in [
        json!({}),
        json!({"ref":{"id":"/w=="}}),
        json!({"ref":{"id":STANDARD.encode(fixed(1,1))}}),
    ] {
        assert_eq!(call("Get", input).await.1["code"], "invalid_argument");
    }
    for method in ["Get", "Start", "Stop"] {
        let (status, body) = call(method, json!({"ref":reference(99)})).await;
        assert_eq!(status, 404);
        assert_eq!(body["code"], "not_found");
    }
    // Tenant remains entity metadata; no caller identity limits these methods.
    for (number, owner) in [(2, 2), (3, 1)] {
        let (status, body) = call("Get", json!({"ref":reference(number)})).await;
        assert_eq!(status, 200);
        assert_eq!(body["tenant"]["id"], STANDARD.encode(fixed(1, owner)));
    }
    assert_eq!(
        call("Start", json!({"ref":reference(2)})).await.1["phase"],
        "PHASE_RUNNING"
    );
    assert_eq!(
        call("Stop", json!({"ref":reference(2)})).await.1["phase"],
        "PHASE_STOPPED"
    );

    let (status, page) = call("List", json!({"size":1})).await;
    assert_eq!(status, 200);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "alpha");
    let cursor = page["next"].as_str().unwrap();
    let (_, page) = call("List", json!({"size":2,"after":cursor})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["name"], "beta");
    assert_eq!(page["items"][1]["name"], "control");
    let (_, page) = call("List", json!({"after":page["next"]})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["name"], "omega");
    assert_eq!(page["items"][1]["name"], "zulu");
    assert!(page["next"].as_str().unwrap_or_default().is_empty());
    assert_eq!(
        call("List", json!({"after":cursor,"reverse":true})).await.0,
        400
    );
    let (_, page) = call("List", json!({"size":1,"reverse":true})).await;
    assert_eq!(page["items"][0]["name"], "zulu");
    let (_, page) = call("List", json!({"reverse":true,"after":page["next"]})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 4);
    assert_eq!(page["items"][0]["name"], "omega");
    assert_eq!(page["items"][3]["name"], "alpha");
    let (_, page) = call(
        "List",
        json!({"filters":[{"ref":reference(1)},{"ref":reference(2)}]}),
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["name"], "alpha");
    assert_eq!(page["items"][1]["name"], "beta");
    let (_, page) = call("List", json!({"filters":[{"ref":reference(2)}]})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "beta");
    for input in [
        json!({"filters":[{}]}),
        json!({"filters":vec![json!({"ref":reference(1)});65]}),
        json!({"after":"invalid"}),
    ] {
        assert_eq!(call("List", input).await.0, 400);
    }
    for size in [-1, 0, 1001] {
        assert_eq!(
            call("List", json!({"size":size})).await.1["items"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
    }
    let (_, body) = call("Get", json!({"ref":reference(3),"select":{"name":true}})).await;
    assert_eq!(body["id"], reference(3)["id"]);
    assert_eq!(body["name"], "control");
    assert!(body["tenant"].is_null());
    assert!(body["manifest"].is_null());
    let (_, body) = call("Get", json!({"ref":reference(3),"select":{"tenant":{}}})).await;
    assert_eq!(body["tenant"]["id"], STANDARD.encode(fixed(1, 1)));
    assert_eq!(
        call(
            "Get",
            json!({"ref":reference(3),"select":{"tenant":{"all":true}}})
        )
        .await
        .0,
        501
    );

    let (status, body) = call("Start", json!({"ref":reference(3)})).await;
    assert_eq!(status, 200);
    assert_eq!(body["phase"], "PHASE_RUNNING");
    assert_eq!(body["id"], reference(3)["id"]);
    let app = h.instance("control");
    let lease = runner.acquire(&app).await.unwrap();
    assert_eq!(
        call("Stop", json!({"ref":reference(3)})).await.1["code"],
        "failed_precondition"
    );
    drop(lease);
    assert_eq!(
        call("Stop", json!({"ref":reference(3)})).await.1["phase"],
        "PHASE_STOPPED"
    );
    let request = http::Request::builder()
        .method("POST")
        .version(http::Version::HTTP_2)
        .uri("/wafl.InstanceService/List")
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(Full::new(Bytes::from_static(&[0, 0, 0, 0, 0])))
        .unwrap();
    let response = service.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap();
    assert_eq!(body.trailers().unwrap()["grpc-status"], "0");
    let bytes = body.to_bytes();
    let list =
        wafl_runner::proto::wafl::InstanceListResponse::decode_from_slice(&bytes[5..]).unwrap();
    assert_eq!(list.items.len(), 5);
    assert_eq!(list.items[1].tenant.id, fixed(1, 2));
    assert_eq!(list.items[2].id, fixed(7, 3));
    assert_eq!(list.items[2].tenant.id, fixed(1, 1));
    assert_eq!(list.items[2].phase, Phase::Stopped);
    assert_eq!(call("Erase", reference(1)).await.1["erased"], true);
    // A cursor is a position, even if that instance was removed between pages.
    let (_, page) = call("List", json!({"after":cursor})).await;
    assert_eq!(page["items"][0]["name"], "beta");
    runner.shutdown().await;
    assert!(runner.acquire(&app).await.is_err());
    h.finish().await;
}

async fn rpc(
    runner: &Runner,
    method: &str,
    body: serde_json::Value,
) -> (http::StatusCode, serde_json::Value) {
    use http_body_util::Full;
    use tower::ServiceExt;
    let service = connectrpc::ConnectRpcService::new(wafl_runner::control::router(runner.clone()));
    let request = http::Request::builder()
        .method("POST")
        .uri(format!("/wafl.InstanceService/{method}"))
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1")
        .body(Full::new(Bytes::from(serde_json::to_vec(&body).unwrap())))
        .unwrap();
    let response = service.oneshot(request).await.unwrap();
    (
        response.status(),
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn generated_add_runs_bundles_and_erase_releases_the_instance() {
    use serde_json::json;
    let mut h = Harness::new(vec![]).await;
    let spec = app("dynamic", "socket");
    let assets = Assets::new([("bundle".into(), spec.bundle())].into()).await;
    let request = serde_json::to_value(spec.request(assets.url("bundle"))).unwrap();
    let (status, body) = rpc(&h.runner, "Add", request.clone()).await;
    assert_eq!(status, 200, "{body}");
    let instance: Instance = serde_json::from_value(body).unwrap();
    assert_eq!(instance.id, spec.id);
    assert_eq!(instance.tenant.id, spec.tenant);
    assert_eq!(instance.phase, Phase::Running);
    assert_eq!(instance.manifest.runtime, "wasmtime");
    assert_eq!(instance.manifest.port, 8080);
    assert_eq!(
        rpc(&h.runner, "Add", request).await.1["code"],
        "already_exists"
    );
    // The same bundle URL and name can create an independent, server-minted ID.
    let mut second = spec.request(assets.url("bundle"));
    second.id = None;
    let (status, body) = rpc(&h.runner, "Add", serde_json::to_value(second).unwrap()).await;
    assert_eq!(status, 200, "{body}");
    let second: Instance = serde_json::from_value(body).unwrap();
    assert!(paydirt::has_domain(&second.id, 7));
    assert_ne!(second.id, instance.id);
    h.ids.insert("dynamic".into(), spec.id);
    h.ids.insert("second".into(), second.id.try_into().unwrap());
    let mut client = h.client().await;
    assert_eq!(client.text("dynamic", "/").await, "dynamic:1");
    assert_eq!(client.text("second", "/").await, "dynamic:1");
    assert_eq!(client.text("dynamic", "/data").await, "<h1>dynamic</h1>");
    assert_eq!(client.text("dynamic", "/headers").await, "preserved");
    assert_eq!(client.text("dynamic", "/denied").await, "denied");
    let reference = json!({"id": STANDARD.encode(spec.id)});
    let (_, selected) = rpc(
        &h.runner,
        "Get",
        json!({"ref":reference,"select":{"manifest":true}}),
    )
    .await;
    assert_eq!(selected["manifest"]["bundle"]["url"], assets.url("bundle"));
    assert!(selected["tenant"].is_null());
    // A stopped instance restarts from its retained files, without a download.
    drop(assets);
    assert_eq!(
        rpc(&h.runner, "Stop", json!({"ref":reference})).await.0,
        200
    );
    assert_eq!(client.text("dynamic", "/data").await, "<h1>dynamic</h1>");
    let execution = h.instance("dynamic");
    let lease = h.runner.acquire(&execution).await.unwrap();
    assert_eq!(
        rpc(&h.runner, "Erase", reference.clone()).await.1["code"],
        "failed_precondition"
    );
    drop(lease);
    assert_eq!(
        rpc(&h.runner, "Erase", reference.clone()).await.1["erased"],
        true
    );
    assert_eq!(rpc(&h.runner, "Erase", reference).await.1["erased"], false);
    assert!(h.runner.acquire(&execution).await.is_err());
    assert_eq!(client.send("dynamic", "/").await.status(), 404);
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_add_releases_its_id_and_shutdown_interrupts_downloads() {
    let directory = tempfile::tempdir().unwrap();
    let runner = Runner::new(&Config {
        directory: Some(directory.path().to_owned()),
        ..config()
    })
    .await
    .unwrap();
    let spec = app("cancelled", "socket");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/stalled", listener.local_addr().unwrap());
    let request = spec.request(url.clone());
    let task = {
        let runner = runner.clone();
        let request = request.clone();
        tokio::spawn(async move { runner.add(request).await })
    };
    let (_connection, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(runner.list().len(), 1);
    assert_eq!(runner.list()[0].phase, Phase::Preparing);
    let reference = serde_json::json!({"id": STANDARD.encode(spec.id)});
    assert_eq!(
        rpc(&runner, "Get", serde_json::json!({"ref":reference}))
            .await
            .1["phase"],
        "PHASE_PREPARING"
    );
    for method in ["Start", "Stop", "Drain"] {
        assert_eq!(
            rpc(&runner, method, serde_json::json!({"ref":reference}))
                .await
                .1["code"],
            "failed_precondition"
        );
    }
    assert_eq!(
        rpc(&runner, "Erase", reference).await.1["code"],
        "failed_precondition"
    );
    assert_eq!(
        runner.add(request).await.unwrap_err().code,
        connectrpc::ErrorCode::AlreadyExists
    );
    task.abort();
    let _ = task.await;
    let assets = Assets::new([("bundle".into(), spec.bundle())].into()).await;
    runner
        .add(spec.request(assets.url("bundle")))
        .await
        .unwrap();
    runner.erase(&spec.id).await.unwrap();
    let task = {
        let runner = runner.clone();
        let request = spec.request(url);
        tokio::spawn(async move { runner.add(request).await })
    };
    let (_connection, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
        .await
        .unwrap()
        .unwrap();
    runner.shutdown().await;
    let error = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.code, connectrpc::ErrorCode::Unavailable);
    assert!(runner.list().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejected_downloads_and_bundles_leave_no_instance() {
    let directory = tempfile::tempdir().unwrap();
    let runner = Runner::new(&Config {
        directory: Some(directory.path().to_owned()),
        ..config()
    })
    .await
    .unwrap();
    let spec = app("invalid", "socket");
    let assets = Assets::new(
        [
            ("gzip".into(), b"not gzip".to_vec()),
            (
                "missing".into(),
                archive(&[("data/index.html", b"hello".to_vec())]),
            ),
            (
                "component".into(),
                archive(&[("main.wasm", b"not wasm".to_vec())]),
            ),
        ]
        .into(),
    )
    .await;
    for (path, code) in [
        ("absent", "unavailable"),
        ("gzip", "invalid_argument"),
        ("missing", "invalid_argument"),
        ("component", "invalid_argument"),
    ] {
        let (_, error) = rpc(
            &runner,
            "Add",
            serde_json::to_value(spec.request(assets.url(path))).unwrap(),
        )
        .await;
        assert_eq!(error["code"], code, "{error}");
        assert!(runner.list().is_empty());
    }
    runner.shutdown().await;
}

#[test]
fn process_config_and_manifest_validation() {
    assert!(config().validate().is_ok());
    assert!(
        serde_json::from_value::<Config>(
            serde_json::json!({"listen":"127.0.0.1:8080","control":"127.0.0.1:8081","apps":[]})
        )
        .is_err()
    );
    let value = app("test", "socket");
    let request = value.request("http://127.0.0.1/bundle.tar.gz".into());
    assert!(wafl_runner::config::Spec::parse(request.clone()).is_ok());
    for url in [
        "file://remote-host/tmp/a",
        "ftp://host/a",
        "http://user:password@host/a",
        "http://host/a#fragment",
    ] {
        assert!(wafl_runner::config::Spec::parse(value.request(url.into())).is_err());
    }
    let mut bad = request.clone();
    bad.phase = Phase::Running.into();
    assert!(wafl_runner::config::Spec::parse(bad).is_err());
    let mut bad = request.clone();
    bad.id = Some(fixed(1, 1).to_vec());
    assert!(wafl_runner::config::Spec::parse(bad).is_err());
    let mut bad = request;
    bad.manifest = Default::default();
    assert!(wafl_runner::config::Spec::parse(bad).is_err());
}

async fn backend(reply: &'static [u8]) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = [0; 4];
        socket.read_exact(&mut bytes).await.unwrap();
        assert_eq!(&bytes, b"ping");
        socket.write_all(reply).await.unwrap();
    });
    (address, task)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn service_bindings_map_the_same_guest_port_for_each_app() {
    let (a, a_task) = backend(b"db-a").await;
    let (b, b_task) = backend(b"db-b").await;
    let mut alpha = app("alpha", "socket");
    alpha.bindings.push(Binding {
        port: 5432,
        target: a.to_string(),
        ..Default::default()
    });
    let mut beta = app("beta", "socket");
    beta.bindings.push(Binding {
        port: 5432,
        target: b.to_string(),
        ..Default::default()
    });
    let h = Harness::new(vec![alpha, beta]).await;
    let mut client = h.client().await;
    assert_eq!(client.text("alpha", "/db").await, "db-a");
    assert_eq!(client.text("beta", "/db").await, "db-b");
    a_task.await.unwrap();
    b_task.await.unwrap();
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn outbound_tcp_resolves_names_and_connects_without_bindings() {
    // Local endpoints make this independent of public DNS and Internet access.
    let (ipv4, ipv4_task) = backend(b"ipv4").await;
    let (dns, dns_task) = backend(b"dns!").await;
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let ipv6 = listener.local_addr().unwrap();
    let ipv6_task = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut bytes = [0; 4];
            socket.read_exact(&mut bytes).await.unwrap();
            assert_eq!(&bytes, b"ping");
            socket.write_all(b"ipv6").await.unwrap();
        }
    });
    let (bound, bound_task) = backend(b"bind").await;
    let apps = [
        ("ipv4", ipv4.to_string()),
        ("dns", format!("localhost:{}", dns.port())),
        ("ipv6", ipv6.to_string()),
        ("bound", bound.to_string()),
    ]
    .into_iter()
    .map(|(name, address)| {
        let mut spec = app(name, "socket");
        spec.env.insert("BACKEND".into(), address);
        assert!(spec.bindings.is_empty());
        spec
    })
    .collect();
    let h = Harness::new(apps).await;
    let mut client = h.client().await;
    assert_eq!(client.text("ipv4", "/outbound").await, "ipv4");
    assert_eq!(client.text("dns", "/outbound").await, "dns!");
    assert_eq!(client.text("ipv6", "/outbound").await, "ipv6");
    assert_eq!(client.text("bound", "/bound").await, "bind");
    assert_eq!(client.text("ipv6", "/bound").await, "ipv6");
    ipv4_task.await.unwrap();
    dns_task.await.unwrap();
    ipv6_task.await.unwrap();
    bound_task.await.unwrap();
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires Redis; set WAFL_TEST_DATABASE_URL and run with --ignored"]
async fn redis_connection_string_keeps_state_across_instances_and_restarts() {
    let database = std::env::var("WAFL_TEST_DATABASE_URL").expect("Redis connection string");
    let key = format!(
        "wafl:test:{}",
        paydirt::format_id(&paydirt::mint(7)).unwrap()
    );
    let apps = ["alpha", "beta"]
        .into_iter()
        .map(|name| {
            let mut spec = app(name, "wafl-example-redis");
            spec.env.insert("DATABASE_URL".into(), database.clone());
            spec.env.insert("COUNTER_KEY".into(), key.clone());
            assert!(spec.bindings.is_empty());
            spec
        })
        .collect();
    let h = Harness::new(apps).await;
    let mut client = h.client().await;
    assert_eq!(client.text("alpha", "/").await, "visits: 1\n");
    let alpha = h.instance("alpha");
    alpha.stop(false, false).await.unwrap();
    assert_eq!(alpha.snapshot().phase, Phase::Stopped);
    assert_eq!(client.text("alpha", "/").await, "visits: 2\n");
    assert_eq!(alpha.snapshot().generation, 2);
    assert_eq!(client.text("beta", "/").await, "visits: 3\n");
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grpc_bidirectional_data_trailers_and_cancellation_survive_the_proxy() {
    let mut spec = app("grpc", "grpc");
    spec.protocol = "h2c".into();
    let h = Harness::new(vec![spec]).await;
    let (mut client, connection) =
        h2::client::handshake(TcpStream::connect(h.address).await.unwrap())
            .await
            .unwrap();
    let driver = tokio::spawn(connection);
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.test/echo.Echo/Listen")
        .header(
            proxy::INSTANCE_HEADER,
            paydirt::format_id(&h.ids["grpc"]).unwrap(),
        )
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(())
        .unwrap();
    let (response, mut output) = client.send_request(request, false).unwrap();
    let payload = Bytes::from_static(b"\x00\x00\x00\x00\x05\x0a\x03abc");
    output.send_data(payload.clone(), false).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(10), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.headers()["content-type"], "application/grpc");
    let mut input = response.into_body();
    let data = tokio::time::timeout(Duration::from_secs(5), input.data())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(data, payload);
    input.flow_control().release_capacity(data.len()).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    h.runner.reap().await;
    let app = h.instance("grpc");
    assert_eq!(app.snapshot().phase, Phase::Running);
    assert_eq!(app.snapshot().active, 1);
    let drain = {
        let app = app.clone();
        tokio::spawn(async move {
            app.drain(5000, false, wafl_runner::runner::Condition::default())
                .await
        })
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !app.snapshot().draining {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(!drain.is_finished());
    output.send_data(payload.clone(), true).unwrap();
    let data = tokio::time::timeout(Duration::from_secs(5), input.data())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(data, payload);
    input.flow_control().release_capacity(data.len()).unwrap();
    assert!(input.data().await.is_none());
    assert_eq!(input.trailers().await.unwrap().unwrap()["grpc-status"], "0");
    drain.await.unwrap().unwrap();
    assert_eq!(app.snapshot().phase, Phase::Stopped);
    drop(
        h.runner
            .start(&app, wafl_runner::runner::Condition::default())
            .await
            .unwrap(),
    );
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.test/echo.Echo/Listen")
        .header(
            proxy::INSTANCE_HEADER,
            paydirt::format_id(&h.ids["grpc"]).unwrap(),
        )
        .body(())
        .unwrap();
    let (response, mut output) = client.send_request(request, false).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), response)
        .await
        .unwrap()
        .unwrap();
    output.send_reset(h2::Reason::CANCEL);
    drop(response);
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(app.snapshot().active, 0);
    h.runner.reap().await;
    assert_eq!(app.snapshot().phase, Phase::Stopped);
    driver.abort();
    h.finish().await;
}

#[path = "runner/lifecycle.rs"]
mod lifecycle;

#[path = "runner/direct.rs"]
mod direct;
