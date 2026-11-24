use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper_util::rt::TokioIo;
use paydirt::spin::{Lease, Workers};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    task::JoinHandle,
};
use wafl_runner::{
    config::{App, Binding, Config, Protocol},
    proto::wafl::Phase,
    proxy,
    runner::Runner,
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
fn app(name: &str, artifact: &str) -> App {
    serde_json::from_value(serde_json::json!({
        "id":paydirt::format_id(&paydirt::mint(7)).unwrap(), "tenant":"00000000-0000-8000-8001-000000000001",
        "name":name, "artifact":guest(artifact), "runtime":"wasmtime", "abi":"wasip2",
        "domain":format!("{name}.test"), "port":8080, "idle_ms":100,
        "env":{"GREETING":name}
    }))
    .unwrap()
}
fn config(apps: Vec<App>) -> Config {
    Config {
        listen: "127.0.0.1:0".parse().unwrap(),
        control: "127.0.0.1:0".parse().unwrap(),
        apps,
    }
}

struct Harness {
    runner: Runner,
    address: SocketAddr,
    stop: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}
impl Harness {
    async fn new(apps: Vec<App>) -> Self {
        let runner = Runner::new(&config(apps)).await.unwrap();
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
            stop: Some(stop),
            task,
        }
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
}
impl Client {
    async fn new(address: SocketAddr) -> Self {
        let (sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(
            TcpStream::connect(address).await.unwrap(),
        ))
        .await
        .unwrap();
        Self {
            sender,
            driver: tokio::spawn(async {
                let _ = connection.await;
            }),
        }
    }
    async fn send(&mut self, domain: &str, path: &str) -> hyper::Response<hyper::body::Incoming> {
        self.sender.ready().await.unwrap();
        let request = hyper::Request::builder()
            .uri(path)
            .header("host", domain)
            .body(Empty::new())
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), self.sender.send_request(request))
            .await
            .unwrap()
            .unwrap()
    }
    async fn text(&mut self, domain: &str, path: &str) -> String {
        let response = self.send(domain, path).await;
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
    assert!(h.runner.list().iter().all(|i| i.phase == Phase::Stopped));
    let mut client = Client::new(h.address).await;
    assert_eq!(client.text("ALPHA.test:80", "/").await, "alpha:1");
    assert_eq!(client.text("beta.test", "/").await, "beta:1");
    assert_eq!(client.text("alpha.test", "/").await, "alpha:2");
    assert_eq!(client.text("alpha.test", "/denied").await, "denied");
    let unknown = client.send("unknown.test", "/").await;
    assert_eq!(unknown.status(), 404);
    unknown.into_body().collect().await.unwrap();
    let alpha = h.runner.app("alpha").unwrap();
    assert_eq!(alpha.snapshot().generation, 1);
    let response = client.send("alpha.test", "/slow").await;
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
    assert_eq!(client.text("alpha.test", "/").await, "alpha:1");
    assert_eq!(alpha.snapshot().generation, 2);
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn simultaneous_cold_requests_and_cancelled_starters_use_one_instance() {
    let mut spec = app("cold", "socket");
    spec.env.insert("MODE".into(), "delay".into());
    let h = Harness::new(vec![spec]).await;
    let app = h.runner.app("cold").unwrap();
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
        requests.spawn(async move { Client::new(address).await.text("cold.test", "/").await });
    }
    let mut responses = std::collections::BTreeSet::new();
    while let Some(result) = requests.join_next().await {
        responses.insert(result.unwrap());
    }
    assert_eq!(responses.len(), 16);
    assert_eq!(app.snapshot().generation, 1);
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
    let mut spin = app("spin", "socket");
    spin.env.insert("MODE".into(), "spin".into());
    spin.startup_ms = 120;
    let mut exit = app("exit", "socket");
    exit.env.insert("MODE".into(), "exit".into());
    let mut memory = app("memory", "socket");
    memory.env.insert("MODE".into(), "memory".into());
    memory.memory_bytes = 2 * 1024 * 1024;
    let mut closed = app("closed", "socket");
    closed.env.insert("MODE".into(), "close".into());
    let mut good = app("good", "socket");
    good.max_active = 1;
    let h = Harness::new(vec![spin, exit, memory, closed, good]).await;
    for name in ["spin", "exit", "memory"] {
        let app = h.runner.app(name).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(3), h.runner.acquire(&app))
                .await
                .unwrap()
                .is_err()
        );
        assert_eq!(app.snapshot().phase, Phase::Failed);
        assert_eq!(app.snapshot().active, 0);
        assert!(!app.snapshot().error.is_empty());
    }
    let good = h.runner.app("good").unwrap();
    let closed = h.runner.app("closed").unwrap();
    let _ = h.runner.acquire(&closed).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(closed.snapshot().phase, Phase::Failed);
    let lease = h.runner.acquire(&good).await.unwrap();
    assert!(h.runner.acquire(&good).await.is_err());
    drop(lease);
    assert_eq!(
        Client::new(h.address).await.text("good.test", "/").await,
        "good:1"
    );
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn management_contract_authentication_and_lifecycle() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use buffa::Message;
    use http_body_util::Full;
    use serde_json::json;
    use tower::ServiceExt;
    use wafl_runner::config::Id;
    fn fixed(domain: u8, number: u8) -> Id {
        Id([
            0, 0, 0, 0, 0, 0, 0x80, 0, 0x80, domain, 0, 0, 0, 0, 0, number,
        ])
    }
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
    let runner = Runner::new(&config(apps)).await.unwrap();
    let credentials = wafl_runner::control::tokens(
        r#"{
        "test-a":"00000000-0000-8000-8001-000000000001",
        "test-b":"00000000-0000-8000-8001-000000000002"
    }"#,
    )
    .unwrap();
    let service = connectrpc::ConnectRpcService::new(wafl_runner::control::router(
        runner.clone(),
        credentials,
    ));
    let call = |method: &str, token: Option<&str>, body: serde_json::Value| {
        let service = service.clone();
        let mut request = http::Request::builder()
            .method("POST")
            .uri(format!("/wafl.InstanceService/{method}"))
            .header("content-type", "application/json")
            .header("connect-protocol-version", "1")
            // Untrusted scope headers must not override the token's tenant.
            .header("payday-tenant", "00000000-0000-8000-8001-000000000002");
        if let Some(token) = token {
            request = request.header("authorization", token);
        }
        let request = request
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
    let reference = |number| json!({"id":STANDARD.encode(fixed(7,number).0)});
    let a = Some("Bearer test-a");
    let b = Some("Bearer test-b");
    for method in ["Get", "List", "Start", "Stop"] {
        for token in [None, Some("Bearer unknown")] {
            let (status, body) = call(method, token, json!({})).await;
            assert_eq!(status, 401);
            assert_eq!(body["code"], "unauthenticated");
        }
    }
    for input in [
        json!({}),
        json!({"ref":{"id":"/w=="}}),
        json!({"ref":{"id":STANDARD.encode(fixed(1,1).0)}}),
    ] {
        assert_eq!(call("Get", a, input).await.1["code"], "invalid_argument");
    }
    for method in ["Get", "Start", "Stop"] {
        let foreign = call(method, a, json!({"ref":reference(2)})).await;
        let absent = call(method, a, json!({"ref":reference(99)})).await;
        assert_eq!(foreign.0, 404);
        assert_eq!(foreign, absent);
        assert_eq!(call(method, b, json!({"ref":reference(3)})).await.0, 404);
    }
    assert_eq!(runner.app("beta").unwrap().snapshot().generation, 0);
    assert_eq!(runner.app("control").unwrap().snapshot().generation, 0);

    let (status, page) = call("List", a, json!({"size":1})).await;
    assert_eq!(status, 200);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "alpha");
    let cursor = page["next"].as_str().unwrap();
    let (_, page) = call("List", a, json!({"size":2,"after":cursor})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["name"], "control");
    assert_eq!(page["items"][1]["name"], "zulu");
    assert!(page["next"].as_str().unwrap_or_default().is_empty());
    assert_eq!(call("List", b, json!({"after":cursor})).await.0, 400);
    assert_eq!(
        call("List", a, json!({"after":cursor,"reverse":true}))
            .await
            .0,
        400
    );
    let (_, page) = call("List", a, json!({"size":1,"reverse":true})).await;
    assert_eq!(page["items"][0]["name"], "zulu");
    let (_, page) = call("List", a, json!({"reverse":true,"after":page["next"]})).await;
    assert_eq!(page["items"][0]["name"], "control");
    assert_eq!(page["items"][1]["name"], "alpha");
    let (_, page) = call(
        "List",
        a,
        json!({"filters":[{"ref":reference(1)},{"ref":reference(2)}]}),
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "alpha");
    let (_, page) = call("List", a, json!({"filters":[{"ref":reference(2)}]})).await;
    assert!(
        page["items"]
            .as_array()
            .is_none_or(|items| items.is_empty())
    );
    let (_, page) = call("List", b, json!({})).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["name"], "beta");
    for input in [
        json!({"filters":[{}]}),
        json!({"filters":vec![json!({"ref":reference(1)});65]}),
        json!({"after":"invalid"}),
    ] {
        assert_eq!(call("List", a, input).await.0, 400);
    }
    for size in [-1, 0, 1001] {
        assert_eq!(
            call("List", a, json!({"size":size})).await.1["items"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }
    let (_, body) = call("Get", a, json!({"ref":reference(3),"select":{"name":true}})).await;
    assert_eq!(body["id"], reference(3)["id"]);
    assert_eq!(body["name"], "control");
    assert!(body["tenant"].is_null());
    assert!(body["domain"].as_str().unwrap_or_default().is_empty());
    let (_, body) = call("Get", a, json!({"ref":reference(3),"select":{"tenant":{}}})).await;
    assert_eq!(body["tenant"]["id"], STANDARD.encode(fixed(1, 1).0));
    assert_eq!(
        call(
            "Get",
            a,
            json!({"ref":reference(3),"select":{"tenant":{"all":true}}})
        )
        .await
        .0,
        501
    );

    let (status, body) = call("Start", a, json!({"ref":reference(3)})).await;
    assert_eq!(status, 200);
    assert_eq!(body["phase"], "PHASE_RUNNING");
    assert_eq!(body["id"], reference(3)["id"]);
    let app = runner.app("control").unwrap();
    let lease = runner.acquire(&app).await.unwrap();
    assert_eq!(
        call("Stop", a, json!({"ref":reference(3)})).await.1["code"],
        "failed_precondition"
    );
    drop(lease);
    assert_eq!(
        call("Stop", a, json!({"ref":reference(3)})).await.1["phase"],
        "PHASE_STOPPED"
    );
    let request = http::Request::builder()
        .method("POST")
        .version(http::Version::HTTP_2)
        .uri("/wafl.InstanceService/List")
        .header("authorization", "Bearer test-a")
        .header("content-type", "application/grpc")
        .header("te", "trailers")
        .body(Full::new(Bytes::from_static(&[0, 0, 0, 0, 0])))
        .unwrap();
    let response = service.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap();
    assert_eq!(body.trailers().unwrap()["grpc-status"], "0");
    let bytes = body.to_bytes();
    let list =
        wafl_runner::proto::wafl::InstanceListResponse::decode_from_slice(&bytes[5..]).unwrap();
    assert_eq!(list.items.len(), 3);
    assert_eq!(list.items[1].id, fixed(7, 3).0);
    assert!(
        list.items
            .iter()
            .all(|item| item.tenant.id == fixed(1, 1).0)
    );
    assert_eq!(list.items[1].phase, Phase::Stopped);
    runner.shutdown().await;
    assert!(runner.acquire(&app).await.is_err());
}

#[test]
fn manifest_resolves_artifacts_and_rejects_ambiguous_configuration() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("app.wasm"), []).unwrap();
    let file = directory.path().join("runner.json");
    let mut value = serde_json::json!({"listen":"127.0.0.1:8080", "control":"127.0.0.1:8081", "apps":[{
        "id":"00000000-0000-8000-8007-000000000001", "tenant":"00000000-0000-8000-8001-000000000001",
        "name":"app", "artifact":"app.wasm", "runtime":"wasmtime", "abi":"wasip2", "domain":"app.test", "port":8080
    }]});
    std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
    let parsed = Config::read(&file).unwrap();
    assert_eq!(parsed.apps[0].artifact, directory.path().join("app.wasm"));
    value["apps"][0]["abi"] = "wasix".into();
    assert!(serde_json::from_value::<Config>(value).is_err());
    let mut bad = parsed.clone();
    bad.apps.push(bad.apps[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = parsed.clone();
    bad.apps[0].bindings.push(Binding {
        port: 8080,
        target: "127.0.0.1:5432".parse().unwrap(),
    });
    assert!(bad.validate().is_err());
    let mut bad = parsed.clone();
    bad.apps[0].id.0[9] = 1;
    assert!(bad.validate().is_err());
    let mut bad = parsed.clone();
    bad.apps[0].tenant.0[9] = 7;
    assert!(bad.validate().is_err());
    assert!(wafl_runner::control::tokens("{}").is_err());
    assert!(
        wafl_runner::control::tokens(r#"{"secret":"00000000-0000-8000-8007-000000000001"}"#)
            .is_err()
    );
    let mut bad = parsed;
    bad.apps[0].domain = "app.test:80".into();
    assert!(bad.validate().is_err());
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
async fn service_bindings_are_private_to_each_app() {
    let (a, a_task) = backend(b"db-a").await;
    let (b, b_task) = backend(b"db-b").await;
    let mut alpha = app("alpha", "socket");
    alpha.bindings.push(Binding {
        port: 5432,
        target: a,
    });
    let mut beta = app("beta", "socket");
    beta.bindings.push(Binding {
        port: 5432,
        target: b,
    });
    let h = Harness::new(vec![alpha, beta]).await;
    let mut client = Client::new(h.address).await;
    assert_eq!(client.text("alpha.test", "/db").await, "db-a");
    assert_eq!(client.text("beta.test", "/db").await, "db-b");
    a_task.await.unwrap();
    b_task.await.unwrap();
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grpc_bidirectional_data_trailers_and_cancellation_survive_the_proxy() {
    let mut spec = app("grpc", "grpc");
    spec.protocol = Protocol::H2c;
    let h = Harness::new(vec![spec]).await;
    let (mut client, connection) =
        h2::client::handshake(TcpStream::connect(h.address).await.unwrap())
            .await
            .unwrap();
    let driver = tokio::spawn(connection);
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.test/echo.Echo/Listen")
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
    let app = h.runner.app("grpc").unwrap();
    assert_eq!(app.snapshot().phase, Phase::Running);
    assert_eq!(app.snapshot().active, 1);
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
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.test/echo.Echo/Listen")
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
