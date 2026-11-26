//! Exercise the real Pingora executable, including its connection pool and FIN
//! handling. Kubernetes tests separately cover real WASM and the controller.
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use paydirt_server::axum::{
    self, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::SocketAddr,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use wafl_runner::proto::wafl::{Route, RouteTarget, Routing};

fn id(domain: u8, seed: u8) -> Vec<u8> {
    let mut id = vec![seed; 16];
    id[6] = 0x80;
    id[8] = 0x80;
    id[9] = domain;
    id
}
struct Mock {
    routes: Mutex<Routing>,
    targets: HashMap<Vec<u8>, RouteTarget>,
    observations: Mutex<HashMap<String, Value>>,
    available: AtomicBool,
    wakes: AtomicUsize,
    calls: AtomicUsize,
}
async fn routes(State(s): State<Arc<Mock>>) -> (StatusCode, Json<Value>) {
    if !s.available.load(Ordering::SeqCst) {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({})));
    }
    (
        StatusCode::OK,
        Json(serde_json::to_value(&*s.routes.lock().unwrap()).unwrap()),
    )
}
async fn wake(State(s): State<Arc<Mock>>, Json(body): Json<Value>) -> Json<Value> {
    s.wakes.fetch_add(1, Ordering::SeqCst);
    let app = STANDARD
        .decode(body["ref"]["id"].as_str().unwrap())
        .unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let mut snapshot = s.routes.lock().unwrap();
        for route in &mut snapshot.routes {
            if route.app == app
                && let Some(target) = s.targets.get(&app)
            {
                route.targets = vec![target.clone()];
            }
        }
    });
    Json(json!({}))
}
async fn traffic(
    State(s): State<Arc<Mock>>,
    Path(method): Path<String>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let mut records = s.observations.lock().unwrap();
    if method == "Add" {
        let key = body["id"].as_str().unwrap().to_owned();
        let waiting = body.get("app").is_some();
        let record = records.entry(key).or_insert_with(|| {
            let mut v = body.clone();
            v["activation"] = json!(waiting);
            v["active"] = json!(if waiting { 1 } else { 0 });
            v["started"] = json!(if waiting { "1" } else { "0" });
            v["closed"] = json!(false);
            v
        });
        return (StatusCode::OK, Json(record.clone()));
    }
    let key = body["ref"]["id"].as_str().unwrap();
    let Some(record) = records.get_mut(key) else {
        return (StatusCode::NOT_FOUND, Json(json!({})));
    };
    if method == "Report" {
        if record["closed"] == true && record["active"] == 0 {
            assert_eq!(body["active"], 0, "a closed gate reopened");
        }
        for key in ["sequence", "active", "started", "busyMs", "closed"] {
            record[key] = body[key].clone();
        }
    }
    (StatusCode::OK, Json(record.clone()))
}
fn free() -> SocketAddr {
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    socket.local_addr().unwrap()
}
struct Proxy(Child);
impl Drop for Proxy {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn until(mut condition: impl AsyncFnMut() -> bool) {
    for _ in 0..100 {
        if condition().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("condition timed out");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pingora_cold_http_pool_generations_tcp_drain_and_staleness() {
    let backend = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http_app = id(10, 1);
    let tcp_app = id(10, 2);
    let mut initial = Routing {
        version: 1,
        routes: vec![
            Route {
                app: http_app.clone(),
                tenant: "default".into(),
                hosts: vec!["proxy.test".into()],
                protocol: "http".into(),
                wait_ms: 3000,
                waiting: 10,
                ..Default::default()
            },
            Route {
                app: tcp_app.clone(),
                tenant: "default".into(),
                port: 10000,
                protocol: "tcp".into(),
                wait_ms: 3000,
                waiting: 10,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    initial.routes.push(Route {
        app: id(10, 4),
        tenant: "default".into(),
        hosts: vec!["waiting.test".into()],
        protocol: "h2c".into(),
        wait_ms: 30_000,
        waiting: 10,
        ..Default::default()
    });
    let s = Arc::new(Mock {
        routes: Mutex::new(initial),
        targets: HashMap::from([
            (
                http_app,
                RouteTarget {
                    placement: id(11, 1),
                    address: backend.local_addr().unwrap().to_string(),
                    ..Default::default()
                },
            ),
            (
                tcp_app,
                RouteTarget {
                    placement: id(11, 2),
                    address: tcp.local_addr().unwrap().to_string(),
                    ..Default::default()
                },
            ),
        ]),
        observations: Mutex::new(HashMap::new()),
        available: AtomicBool::new(true),
        wakes: AtomicUsize::new(0),
        calls: AtomicUsize::new(0),
    });
    let backend_state = s.clone();
    let connections = Arc::new(AtomicUsize::new(0));
    let connection_count = connections.clone();
    tokio::spawn(async move {
        loop {
            let (stream, _) = backend.accept().await.unwrap();
            let number = connection_count.fetch_add(1, Ordering::SeqCst);
            let state = backend_state.clone();
            tokio::spawn(async move {
                let service = hyper::service::service_fn(
                    move |request: hyper::Request<hyper::body::Incoming>| {
                        let state = state.clone();
                        async move {
                            state.calls.fetch_add(1, Ordering::SeqCst);
                            if request.uri().path() == "/fail" {
                                return Err(std::io::Error::other(
                                    "backend committed then disconnected",
                                ));
                            }
                            if request.uri().path() == "/hold" {
                                tokio::time::sleep(Duration::from_secs(60)).await;
                            }
                            let body = request.into_body().collect().await.unwrap().to_bytes();
                            Ok::<_, std::io::Error>(
                                hyper::Response::builder()
                                    .header("x-connection", number.to_string())
                                    .body(Full::new(body))
                                    .unwrap(),
                            )
                        }
                    },
                );
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                    .await;
            });
        }
    });
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = tcp.accept().await.unwrap();
            tokio::spawn(async move {
                stream.write_all(b"ready\n").await.unwrap();
                let mut buf = [0u8; 4096];
                loop {
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    if n == 0 {
                        break;
                    }
                    if stream.write_all(&buf[..n]).await.is_err() {
                        return;
                    }
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
                let _ = stream.write_all(b"done\n").await;
            });
        }
    });
    let control = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", control.local_addr().unwrap());
    let app = Router::new()
        .route("/proxy", get(routes))
        .route("/wafl.AppService/Wake", post(wake))
        .route("/wafl.TrafficService/{method}", post(traffic))
        .with_state(s.clone());
    tokio::spawn(async move {
        axum::serve(control, app).await.unwrap();
    });
    let listen = free();
    let health = free();
    let certificates = tempfile::tempdir().unwrap();
    let certificate = certificates.path().join("cert.pem");
    let key = certificates.path().join("key.pem");
    let tls = free();
    assert!(
        Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=proxy.test",
                "-keyout",
                key.to_str().unwrap(),
                "-out",
                certificate.to_str().unwrap()
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    let mut proxy = Proxy(
        Command::new(env!("CARGO_BIN_EXE_wafl"))
            .args([
                "serve",
                "proxy",
                "--controller",
                &origin,
                "--listen",
                &listen.to_string(),
                "--health",
                &health.to_string(),
                "--tls-listen",
                &tls.to_string(),
                "--tls-cert",
                certificate.to_str().unwrap(),
                "--tls-key",
                key.to_str().unwrap(),
                "--tcp-ip",
                "127.0.0.2",
                "--drain-seconds",
                "1",
            ])
            .env("RUST_LOG", "error")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let health = format!("http://{health}/health");
    let url = format!("http://{listen}");
    until(async || {
        client
            .get(&health)
            .send()
            .await
            .is_ok_and(|r| r.status() == 200)
    })
    .await;
    let payload = Bytes::from(vec![0x8fu8; 80_000]);
    let response = client
        .post(&url)
        .header("host", "proxy.test")
        .body(payload.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let first = response.headers()["x-connection"].clone();
    assert_eq!(response.bytes().await.unwrap(), payload);
    assert_eq!(s.wakes.load(Ordering::SeqCst), 1);
    let response = client
        .get(&url)
        .header("host", "proxy.test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.headers()["x-connection"], first);
    response.bytes().await.unwrap();
    assert_eq!(connections.load(Ordering::SeqCst), 1);
    let secure = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let response = secure
        .get(format!("https://{tls}/"))
        .header("host", "proxy.test")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response.bytes().await.unwrap();
    // Reusing the same address for a different execution must use a new pool.
    s.routes.lock().unwrap().routes[0].targets[0].placement = id(11, 3);
    tokio::time::sleep(Duration::from_millis(650)).await;
    let response = client
        .get(&url)
        .header("host", "proxy.test")
        .send()
        .await
        .unwrap();
    assert_ne!(response.headers()["x-connection"], first);
    response.bytes().await.unwrap();
    let before = s.calls.load(Ordering::SeqCst);
    let _ = client
        .post(format!("{url}/fail"))
        .header("host", "proxy.test")
        .body("once")
        .send()
        .await;
    assert_eq!(
        s.calls.load(Ordering::SeqCst),
        before + 1,
        "proxy replayed a failed request"
    );
    let mut stream = TcpStream::connect("127.0.0.2:10000").await.unwrap();
    let mut greeting = [0u8; 6];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut greeting))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&greeting, b"ready\n");
    assert_eq!(s.wakes.load(Ordering::SeqCst), 2);
    // Withdrawal stops new admissions, but holds the existing TCP connection.
    s.routes.lock().unwrap().routes[1].suspended = true;
    s.routes.lock().unwrap().routes[1].targets.clear();
    until(async || {
        s.observations.lock().unwrap().values().any(|v| {
            v["placement"]["id"] == STANDARD.encode(id(11, 2))
                && v["closed"] == true
                && v["active"] == 1
        })
    })
    .await;
    stream.write_all(b"binary\x00\xff").await.unwrap();
    stream.shutdown().await.unwrap();
    let mut output = vec![];
    tokio::time::timeout(Duration::from_secs(3), stream.read_to_end(&mut output))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(output, b"binary\x00\xffdone\n");
    until(async || {
        s.observations.lock().unwrap().values().any(|v| {
            v["placement"]["id"] == STANDARD.encode(id(11, 2))
                && v["closed"] == true
                && v["active"] == 0
        })
    })
    .await;
    let (mut h2, connection) = h2::client::handshake(TcpStream::connect(listen).await.unwrap())
        .await
        .unwrap();
    let driver = tokio::spawn(connection);
    let request = http::Request::builder()
        .method("POST")
        .uri("http://waiting.test/")
        .header("content-type", "application/grpc")
        .header("grpc-timeout", "100m")
        .body(())
        .unwrap();
    let (deadline, _) = h2.send_request(request, true).unwrap();
    let deadline = tokio::time::timeout(Duration::from_secs(5), deadline)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(deadline.status(), 200);
    assert_eq!(deadline.headers()["grpc-status"], "4");
    let waiting_app = STANDARD.encode(id(10, 4));
    until(async || {
        s.observations
            .lock()
            .unwrap()
            .values()
            .any(|v| v["app"]["id"] == waiting_app && v["closed"] == true && v["active"] == 0)
    })
    .await;
    let request = http::Request::builder()
        .method("POST")
        .uri("http://waiting.test/")
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let (cancelled, mut cancel_stream) = h2.send_request(request, true).unwrap();
    until(async || {
        s.observations
            .lock()
            .unwrap()
            .values()
            .any(|v| v["app"]["id"] == waiting_app && v["active"] == 1 && v["closed"] == false)
    })
    .await;
    cancel_stream.send_reset(h2::Reason::CANCEL);
    drop(cancelled);
    until(async || {
        s.observations
            .lock()
            .unwrap()
            .values()
            .filter(|v| v["app"]["id"] == waiting_app)
            .all(|v| v["closed"] == true && v["active"] == 0)
    })
    .await;
    drop(h2);
    driver.abort();
    let slow_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let hold_url = format!("{url}/hold");
    let held = tokio::spawn(async move {
        slow_client
            .get(hold_url)
            .header("host", "proxy.test")
            .send()
            .await
    });
    until(async || {
        s.observations
            .lock()
            .unwrap()
            .values()
            .any(|v| v["placement"]["id"] == STANDARD.encode(id(11, 3)) && v["active"] == 1)
    })
    .await;
    s.available.store(false, Ordering::SeqCst);
    until(async || {
        client
            .get(&health)
            .send()
            .await
            .is_ok_and(|r| r.status() == 503)
    })
    .await;
    assert_eq!(
        client
            .get(&url)
            .header("host", "proxy.test")
            .send()
            .await
            .unwrap()
            .status(),
        503
    );
    Command::new("kill")
        .args(["-TERM", &proxy.0.id().to_string()])
        .status()
        .unwrap();
    until(async || proxy.0.try_wait().unwrap().is_some()).await;
    assert!(
        held.await.unwrap().is_err(),
        "overdue HTTP stream survived proxy drain"
    );
    assert!(
        s.observations
            .lock()
            .unwrap()
            .values()
            .all(|v| v["closed"] == true && v["active"] == 0),
        "normal shutdown left open observers"
    );
}
