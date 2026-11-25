#![cfg(feature = "client")]
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::{Child, Command},
    sync::Notify,
};
use wafl_runner::proto::wafl::{Instance, Log, Phase};

const CLI: &str = env!("CARGO_BIN_EXE_wafl");
#[path = "cli/autoscale.rs"]
mod autoscale;
#[path = "cli/controller.rs"]
mod controller;
#[path = "cli/kubernetes.rs"]
mod kubernetes;
#[path = "cli/secrets.rs"]
mod secrets;
fn command() -> Command {
    let mut command = Command::new(CLI);
    command
        .env("TOKIO_WORKER_THREADS", "2")
        .env("RAYON_NUM_THREADS", "2");
    command
}
fn bundle() -> Vec<u8> {
    bundle_data(b"shared data")
}
fn bundle_data(data: &[u8]) -> Vec<u8> {
    let base = std::env::var_os("WAFL_TEST_GUEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/wasm32-wasip2/debug")
        });
    let wasm = std::fs::read(base.join("socket.wasm")).expect("run bash scripts/test.sh");
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    for (name, data) in [("main.wasm", wasm), ("data/index.html", data.to_vec())] {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, name, data.as_slice()).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap()
}
struct Assets {
    url: String,
    count: Arc<AtomicUsize>,
    started: Arc<Notify>,
    task: tokio::task::JoinHandle<()>,
}
impl Assets {
    async fn new(stall_first: bool) -> Self {
        use paydirt_server::axum::{self, routing::get};
        let count = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let bytes = Arc::new(bundle());
        let calls = count.clone();
        let notice = started.clone();
        let router = axum::Router::new().route(
            "/bundle",
            get(move || {
                let bytes = bytes.clone();
                let calls = calls.clone();
                let notice = notice.clone();
                async move {
                    let index = calls.fetch_add(1, Ordering::SeqCst);
                    notice.notify_one();
                    if stall_first && index == 0 {
                        std::future::pending::<()>().await;
                    }
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    bytes.as_ref().clone()
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/bundle", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            url,
            count,
            started,
            task,
        }
    }
}
impl Drop for Assets {
    fn drop(&mut self) {
        self.task.abort();
    }
}
struct Server {
    child: Child,
    endpoint: String,
    ingress: SocketAddr,
}
impl Server {
    async fn new(directory: &Path, flag: bool) -> Self {
        Self::configured(directory, flag, None).await
    }
    async fn configured(directory: &Path, flag: bool, metrics: Option<serde_json::Value>) -> Self {
        Self::options(
            directory,
            flag,
            metrics.map(|m| serde_json::json!({"metrics":m})),
        )
        .await
    }
    async fn options(directory: &Path, flag: bool, options: Option<serde_json::Value>) -> Self {
        for _ in 0..5 {
            // Hold both reservations so the kernel cannot return the same port
            // twice. Another process can still win the handoff below; retry only
            // that bind failure and wait for this child's own readiness message.
            let ingress_socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let control_socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let ingress = ingress_socket.local_addr().unwrap();
            let control = control_socket.local_addr().unwrap();
            if let Some(direct) = options.as_ref().and_then(|o| o.get("direct")) {
                let first = direct["first"].as_u64().unwrap();
                let last = direct["last"].as_u64().unwrap();
                if (first..=last).contains(&u64::from(control.port())) {
                    continue;
                }
            }
            let mut command = command();
            command.args(["serve", "runner"]);
            if let Some(options) = &options {
                let mut config = options.clone();
                config["listen"] = serde_json::json!(ingress);
                config["control"] = serde_json::json!(control);
                let path = directory.join("runner-metrics.json");
                std::fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
                command.arg("--config").arg(path);
            }
            if flag {
                command.arg("--directory");
            }
            command.arg(directory).args([
                "--listen",
                &ingress.to_string(),
                "--control",
                &control.to_string(),
            ]);
            command
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            drop((ingress_socket, control_socket));
            let mut child = command.spawn().unwrap();
            let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
            let mut diagnostics = String::new();
            let direct = options.as_ref().is_some_and(|o| o.get("direct").is_some());
            let ingress_label = if direct {
                "direct".into()
            } else {
                ingress.to_string()
            };
            let expected = format!("wafl-runner ingress={ingress_label} control={control}");
            let ready =
                tokio::time::timeout(Duration::from_secs(if direct { 30 } else { 10 }), async {
                    while let Some(line) = stderr.next_line().await.unwrap() {
                        diagnostics.push_str(&line);
                        diagnostics.push('\n');
                        if line == expected {
                            return true;
                        }
                    }
                    false
                })
                .await
                .unwrap_or_else(|_| panic!("runner readiness timeout: {diagnostics}"));
            if ready {
                tokio::spawn(async move {
                    while let Ok(Some(line)) = stderr.next_line().await {
                        eprintln!("runner: {line}");
                    }
                });
                return Self {
                    child,
                    endpoint: format!("http://{control}"),
                    ingress,
                };
            }
            let status = child.wait().await.unwrap();
            assert!(
                diagnostics.contains("Address already in use"),
                "runner exited during startup ({status}): {diagnostics}"
            );
        }
        panic!("could not reserve runner ports after five attempts")
    }
    async fn get(&self, instance: &Instance, path: &str) -> reqwest::Response {
        reqwest::Client::new()
            .get(format!("http://{}{path}", self.ingress))
            .header("wafl-instance", paydirt::format_id(&instance.id).unwrap())
            .send()
            .await
            .unwrap()
    }
    async fn stop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().await.unwrap();
        }
    }
}

#[path = "cli/cache.rs"]
mod cache;
#[path = "cli/direct.rs"]
mod direct;
#[path = "cli/lifecycle.rs"]
mod lifecycle;
#[path = "cli/metrics.rs"]
mod metrics;
async fn cli(endpoint: &str, args: &[&str]) -> Output {
    tokio::time::timeout(
        Duration::from_secs(120),
        command()
            .arg("--endpoint")
            .arg(endpoint)
            .args(args)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap()
}
fn success(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
async fn instance(endpoint: &str, args: &[&str]) -> Instance {
    let mut flags = vec!["--json", "instance"];
    flags.extend_from_slice(args);
    serde_json::from_slice(&success(cli(endpoint, &flags).await)).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cli_alias_logs_and_two_processes_share_complete_bundles() {
    let directory = tempfile::tempdir().unwrap();
    let assets = Assets::new(false).await;
    let mut first = Server::new(directory.path(), false).await;
    let mut second = Server::new(directory.path(), true).await;
    let args = [
        "add",
        &assets.url,
        "--alias",
        "worker",
        "--idle-ms",
        "60000",
    ];
    let (one, two) = tokio::join!(
        instance(&first.endpoint, &args),
        instance(&second.endpoint, &args)
    );
    assert_ne!(one.id, two.id);
    assert_eq!(one.alias, "worker");
    assert_eq!(one.manifest.bundle.digest, two.manifest.bundle.digest);
    assert_eq!(
        assets.count.load(Ordering::SeqCst),
        1,
        "only the cache-lock owner should fetch"
    );
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/objects/sha256"))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/tmp"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        instance(&first.endpoint, &["get", "@worker"]).await.id,
        one.id
    );
    assert!(
        !cli(&first.endpoint, &["instance", "start", "worker"])
            .await
            .status
            .success()
    );
    assert!(
        !cli(
            &first.endpoint,
            &["instance", "add", &assets.url, "--alias", "worker"]
        )
        .await
        .status
        .success()
    );
    let output = cli(&first.endpoint, &["instance", "logs", "@worker"]).await;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("guest boot"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("guest stderr"));
    let mut follower = command()
        .args([
            "--endpoint",
            &first.endpoint,
            "--json",
            "instance",
            "logs",
            "-f",
            "@worker",
            "--limit",
            "1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(follower.stdout.take().unwrap()).lines();
    let line = tokio::time::timeout(Duration::from_secs(10), lines.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let entry: Log = serde_json::from_str(&line).unwrap();
    assert_eq!(entry.generation, 1);
    assert_eq!(
        instance(&first.endpoint, &["get", "@worker"]).await.active,
        0
    );
    assert_eq!(
        instance(&first.endpoint, &["stop", "@worker"]).await.phase,
        Phase::Stopped
    );
    let uuid = paydirt::format_id(&one.id).unwrap();
    let restarted = instance(&first.endpoint, &["start", &uuid]).await;
    assert_eq!(restarted.generation, 2);
    let line = tokio::time::timeout(Duration::from_secs(10), lines.next_line())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::from_str::<Log>(&line).unwrap().generation, 2);
    assert_eq!(
        first.get(&one, "/logs").await.text().await.unwrap(),
        "logged"
    );
    success(cli(&first.endpoint, &["instance", "erase", "@worker"]).await);
    assert!(
        tokio::time::timeout(Duration::from_secs(10), follower.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
    assert_eq!(
        second.get(&two, "/data").await.text().await.unwrap(),
        "shared data"
    );
    let url = assets.url.clone();
    drop(assets);
    first.stop().await;
    let mut fresh = Server::new(directory.path(), false).await;
    assert!(success(cli(&fresh.endpoint, &["instance", "list"]).await).is_empty());
    let three = instance(&fresh.endpoint, &["add", &url]).await;
    assert!(!three.alias.is_empty());
    assert_eq!(three.manifest.bundle.digest, one.manifest.bundle.digest);
    assert_eq!(
        fresh.get(&three, "/data").await.text().await.unwrap(),
        "shared data"
    );
    // A digest hit works even with a different, currently unavailable origin.
    let four = instance(
        &fresh.endpoint,
        &[
            "add",
            "http://127.0.0.1:1/unavailable",
            "--digest",
            &one.manifest.bundle.digest,
        ],
    )
    .await;
    assert_eq!(four.manifest.bundle.digest, one.manifest.bundle.digest);
    #[cfg(unix)]
    {
        let alias = format!("@{}", three.alias);
        let mut follower = command()
            .args([
                "--endpoint",
                &fresh.endpoint,
                "instance",
                "logs",
                "-f",
                &alias,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(follower.stdout.take().unwrap()).lines();
        assert!(
            tokio::time::timeout(Duration::from_secs(10), lines.next_line())
                .await
                .unwrap()
                .unwrap()
                .is_some()
        );
        assert!(
            Command::new("kill")
                .args(["-TERM", &fresh.child.id().unwrap().to_string()])
                .status()
                .await
                .unwrap()
                .success()
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(10), fresh.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(10), follower.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
    }
    fresh.stop().await;
    second.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn killed_download_owner_releases_lock_and_staging_is_recovered() {
    let directory = tempfile::tempdir().unwrap();
    let assets = Assets::new(true).await;
    let mut first = Server::new(directory.path(), false).await;
    let mut second = Server::new(directory.path(), false).await;
    let mut abandoned = command()
        .args([
            "--endpoint",
            &first.endpoint,
            "instance",
            "add",
            &assets.url,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), assets.started.notified())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/tmp"))
            .unwrap()
            .count(),
        1
    );
    let args = ["add", assets.url.as_str()];
    let recovery = instance(&second.endpoint, &args);
    let interrupt = async {
        tokio::time::sleep(Duration::from_millis(150)).await;
        first.stop().await;
    };
    let (result, ()) = tokio::join!(recovery, interrupt);
    assert_eq!(result.phase, Phase::Running);
    assert_eq!(assets.count.load(Ordering::SeqCst), 2);
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/tmp"))
            .unwrap()
            .count(),
        0
    );
    assert!(
        !tokio::time::timeout(Duration::from_secs(10), abandoned.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
    second.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_file_and_digest_validation_are_available_from_cli() {
    let directory = tempfile::tempdir().unwrap();
    let source = tempfile::tempdir().unwrap();
    let path = source.path().join("bundle.tar.gz");
    std::fs::write(&path, bundle()).unwrap();
    let url = reqwest::Url::from_file_path(&path).unwrap();
    let mut server = Server::new(directory.path(), false).await;
    let bad = format!("sha256:{}", "0".repeat(64));
    assert!(
        !cli(
            &server.endpoint,
            &["instance", "add", url.as_str(), "--digest", &bad]
        )
        .await
        .status
        .success()
    );
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/objects/sha256"))
            .unwrap()
            .count(),
        0
    );
    let instance = instance(
        &server.endpoint,
        &["add", url.as_str(), "--alias", "@local"],
    )
    .await;
    assert_eq!(instance.alias, "local");
    let output = success(cli(&server.endpoint, &["instance", "get", "@local"]).await);
    assert_eq!(
        String::from_utf8(output).unwrap(),
        format!("{}\t@local\n", paydirt::format_id(&instance.id).unwrap())
    );
    assert!(instance.manifest.bundle.digest.starts_with("sha256:"));
    assert_eq!(
        STANDARD
            .decode(
                serde_json::to_value(&instance).unwrap()["id"]
                    .as_str()
                    .unwrap()
            )
            .unwrap(),
        instance.id
    );
    server.stop().await;
}
