use super::controller::*;
use super::*;
use k8s_openapi::{
    api::core::v1::Namespace,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
};
use kube::{
    Api, Client, ResourceExt,
    api::{DeleteParams, Patch, PatchParams, PostParams},
    core::{ApiResource, DynamicObject, GroupVersionKind},
};
use serde_json::{Value, json};
use wafl_runner::proto::wafl::PlacementPhase;

fn port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
async fn observed(
    api: &Api<DynamicObject>,
    name: &str,
    predicate: impl Fn(&DynamicObject) -> bool,
) -> DynamicObject {
    tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            let object = api.get(name).await.unwrap();
            if predicate(&object) {
                return object;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("Kubernetes App {name} did not converge"))
}
async fn gone(api: &Api<DynamicObject>, name: &str) {
    tokio::time::timeout(Duration::from_secs(90), async {
        while api.get_opt(name).await.unwrap().is_some() {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .expect("App finalizer did not finish");
}
async fn gateway(root: &Path, controller: &str, entry: u16) -> Child {
    let plugin = root.join("plugins-local/src/github.com/lesomnus/wafl/traefik");
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
    let mut child = Command::new(std::env::var("WAFL_TEST_TRAEFIK").unwrap())
        .current_dir(root)
        .args([
            format!("--entrypoints.web.address=127.0.0.1:{entry}"),
            format!("--providers.http.endpoint={controller}/traefik"),
            "--providers.http.pollinterval=100ms".into(),
            "--providers.providersthrottleduration=100ms".into(),
            "--experimental.localplugins.wafl.modulename=github.com/lesomnus/wafl/traefik".into(),
            "--log.level=ERROR".into(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            assert!(child.try_wait().unwrap().is_none());
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
    .unwrap();
    child
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires disposable WAFL_TEST_KUBECONFIG, PostgreSQL and Traefik"]
async fn crd_replicas_restart_updates_stream_drain_and_zero_activation() {
    // Never infer a developer's current cluster. This test installs a CRD and
    // creates its own namespaces in an explicitly supplied disposable cluster.
    let kubeconfig =
        std::env::var("WAFL_TEST_KUBECONFIG").expect("disposable Kubernetes kubeconfig");
    let connection = kube::Config::from_custom_kubeconfig(
        kube::config::Kubeconfig::read_from(&kubeconfig).unwrap(),
        &Default::default(),
    )
    .await
    .unwrap();
    let client = Client::try_from(connection).unwrap();
    let crds: Api<CustomResourceDefinition> = Api::all(client.clone());
    let crd: Value =
        serde_json::from_str(include_str!("../../deploy/kubernetes/apps.json")).unwrap();
    crds.patch(
        "apps.wafl.opt.td",
        &PatchParams::apply("wafl-test").force(),
        &Patch::Apply(crd),
    )
    .await
    .unwrap();
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let crd = crds.get("apps.wafl.opt.td").await.unwrap();
            if crd
                .status
                .and_then(|s| s.conditions)
                .unwrap_or_default()
                .iter()
                .any(|c| c.type_ == "Established" && c.status == "True")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    let ns = format!("wafl-{}", uuid::Uuid::new_v4().simple());
    let other = format!("{ns}-other");
    let namespaces: Api<Namespace> = Api::all(client.clone());
    for name in [&ns, &other] {
        namespaces
            .create(
                &PostParams::default(),
                &serde_json::from_value(json!({"metadata":{"name":name}})).unwrap(),
            )
            .await
            .unwrap();
    }
    let resource = ApiResource::from_gvk(&GroupVersionKind::gvk("wafl.opt.td", "v1alpha1", "App"));
    let api = Api::<DynamicObject>::namespaced_with(client.clone(), &ns, &resource);
    let unwatched = Api::<DynamicObject>::namespaced_with(client, &other, &resource);
    let assets = Assets::new(false).await;
    let desired = json!({"apiVersion":"wafl.opt.td/v1alpha1","kind":"App","metadata":{"name":"hello"},
        "spec":{"domain":"hello.example","manifest":{"bundle":{"url":assets.url},"env":{"GREETING":"kubernetes"}}}});
    let original = api
        .create(
            &PostParams::default(),
            &serde_json::from_value(desired.clone()).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(original.data["spec"]["manifest"]["port"], 8080);
    unwatched
        .create(
            &PostParams::default(),
            &serde_json::from_value(desired.clone()).unwrap(),
        )
        .await
        .unwrap();
    let invalid = api
        .patch(
            "hello",
            &PatchParams::default(),
            &Patch::Merge(json!({"spec":{"manifest":{"runtime":"invalid"}}})),
        )
        .await
        .unwrap_err();
    assert!(matches!(invalid,kube::Error::Api(e) if e.code==422));
    let invalid = api
        .patch(
            "hello",
            &PatchParams::default(),
            &Patch::Merge(json!({"spec":{"domain":"changed.example"}})),
        )
        .await
        .unwrap_err();
    assert!(matches!(invalid,kube::Error::Api(e) if e.code==422));
    let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").unwrap();
    let (sql, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
        .await
        .unwrap();
    let driver = tokio::spawn(driver);
    let schema = format!("kubernetes_{}", uuid::Uuid::new_v4().simple());
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
    let mut runner=Server::options(&cache,false,Some(json!({"direct":{"listen":"127.0.0.1","advertise":"127.0.0.1","directory":root.path().join("ports"),"first":first,"last":first+31}}))).await;
    let control = port();
    let activation = port();
    let entry = port();
    let config = json!({"database":database.as_str().replace('+',"%20"),"runners":[runner.endpoint],"listen":format!("127.0.0.1:{control}"),
        "interval_ms":100,"probe_ms":200,"fresh_ms":3000,
        "traffic":{"control":format!("http://127.0.0.1:{control}"),"activate":format!("http://127.0.0.1:{activation}"),"listen":format!("127.0.0.1:{activation}"),"interval_ms":100,"fresh_ms":1000},
        "providers":{"kubernetes":{"cluster":"test","kubeconfig":kubeconfig,"interval_ms":500,"namespaces":{&ns:"00000000-0000-8000-8001-000000000001"}}}});
    let mut c1 = Controller::start(root.path(), &config).await;
    let mut second = config.clone();
    second["listen"] = json!("127.0.0.1:0");
    second["traffic"]["listen"] = json!("127.0.0.1:0");
    let mut c2 = Controller::start(root.path(), &second).await;
    let ready = observed(&api, "hello", |o| o.data["status"]["ready"] == 1).await;
    assert_eq!(ready.data["status"]["observedGeneration"], 1);
    let alias = ready.data["status"]["alias"].as_str().unwrap().to_owned();
    let original_app = app(&c1.endpoint, &["get", &format!("@{alias}")]).await;
    assert_eq!(original_app.replicas, 1);
    assert_eq!(placements(&c1.endpoint).await.len(), 1);
    assert!(
        unwatched
            .get("hello")
            .await
            .unwrap()
            .finalizers()
            .is_empty()
    );
    let mut duplicate = desired.clone();
    duplicate["metadata"]["name"] = json!("collision");
    api.create(
        &PostParams::default(),
        &serde_json::from_value(duplicate).unwrap(),
    )
    .await
    .unwrap();
    let rejected = observed(&api, "collision", |o| {
        o.data["status"]["conditions"].as_array().is_some_and(|cs| {
            cs.iter()
                .any(|c| c["type"] == "Accepted" && c["status"] == "False")
        })
    })
    .await;
    // A persistent validation error must not create a status/watch busy loop.
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(
        api.get("collision").await.unwrap().resource_version(),
        rejected.resource_version()
    );
    api.delete("collision", &DeleteParams::default())
        .await
        .unwrap();
    gone(&api, "collision").await;
    assert_eq!(placements(&c1.endpoint).await.len(), 1);
    for (method, request) in [
        ("Patch", json!({"ref":{"alias":alias},"replicas":0})),
        ("Apply", json!({"ref":{"alias":alias},"patch":{}})),
        ("Erase", json!({"alias":alias})),
    ] {
        let (status, body) = rpc(&c1.endpoint, "AppService", method, request).await;
        assert_ne!(status, 200, "{body}");
        assert_eq!(body["code"], "failed_precondition");
    }
    let (status, body) = rpc(
        &c1.endpoint,
        "AppService",
        "Add",
        json!({"alias":"forged","source":{"id":STANDARD.encode(&original_app.source.id)}}),
    )
    .await;
    assert_ne!(status, 200);
    assert_eq!(body["code"], "invalid_argument");
    // Ordinary CLI/API apps remain available alongside the provider.
    let (status,body)=rpc(&c1.endpoint,"AppService","Add",json!({"alias":"ordinary","tenant":{"alias":"local"},"domain":"ordinary.example","manifest":{"port":8080,"bundle":{"url":assets.url}}})).await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = rpc(
        &c1.endpoint,
        "AppService",
        "Erase",
        json!({"alias":"ordinary"}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let mut proxy = gateway(root.path(), &c1.endpoint, entry).await;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let address = format!("http://127.0.0.1:{entry}");
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(
        http.get(&address)
            .header("host", "hello.example")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "kubernetes:1"
    );
    // Both watchers stop: the next initial LIST must recover the missed change.
    c1.stop().await;
    c2.stop().await;
    let updated = api
        .patch(
            "hello",
            &PatchParams::default(),
            &Patch::Merge(json!({"spec":{"name":"renamed"}})),
        )
        .await
        .unwrap();
    c1 = Controller::start(root.path(), &config).await;
    c2 = Controller::start(root.path(), &second).await;
    let updated = observed(&api, "hello", |o| {
        o.data["status"]["observedGeneration"] == updated.metadata.generation.unwrap()
    })
    .await;
    assert_eq!(updated.data["status"]["app"], ready.data["status"]["app"]);
    let renamed = app(&c1.endpoint, &["get", &format!("@{alias}")]).await;
    assert_eq!(renamed.revision, original_app.revision);
    assert_eq!(
        renamed.manifest.bundle.digest,
        original_app.manifest.bundle.digest
    );
    assert_eq!(renamed.name, "renamed");
    // A real gRPC stream proves finalization waits for traffic, not a timer.
    let wasm = std::fs::read(
        PathBuf::from(std::env::var_os("WAFL_TEST_GUEST_DIR").unwrap()).join("grpc.wasm"),
    )
    .unwrap();
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
    let path = root.path().join("grpc.tar.gz");
    std::fs::write(&path, archive.into_inner().unwrap().finish().unwrap()).unwrap();
    let mut grpc = desired.clone();
    grpc["metadata"]["name"] = json!("grpc");
    grpc["spec"]["domain"] = json!("grpc.example");
    grpc["spec"]["manifest"] = json!({"protocol":"h2c","bundle":{"url":reqwest::Url::from_file_path(&path).unwrap().as_str()}});
    api.create(
        &PostParams::default(),
        &serde_json::from_value(grpc).unwrap(),
    )
    .await
    .unwrap();
    let ready_grpc = observed(&api, "grpc", |o| o.data["status"]["ready"] == 1).await;
    let grpc_id = paydirt::parse_id(ready_grpc.data["status"]["app"].as_str().unwrap()).unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (mut h2, driver_h2) = h2::client::handshake(
        tokio::net::TcpStream::connect(("127.0.0.1", entry))
            .await
            .unwrap(),
    )
    .await
    .unwrap();
    let driver_h2 = tokio::spawn(driver_h2);
    let request = http::Request::builder()
        .method("POST")
        .uri("http://grpc.example/echo.Echo/Listen")
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let (reply, mut upload) = h2.send_request(request, false).unwrap();
    let data = bytes::Bytes::from_static(b"\x00\x00\x00\x00\x05\x0a\x03abc");
    upload.send_data(data.clone(), false).unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(30), reply)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reply.status(), 200);
    let mut body = reply.into_body();
    assert_eq!(body.data().await.unwrap().unwrap(), data);
    body.flow_control().release_capacity(data.len()).unwrap();
    api.delete("grpc", &DeleteParams::default()).await.unwrap();
    until(&c1.endpoint, |ps| {
        ps.iter()
            .any(|p| p.app.id == grpc_id && p.phase == PlacementPhase::Retiring)
    })
    .await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let deleting = api.get("grpc").await.unwrap();
    assert!(deleting.metadata.deletion_timestamp.is_some());
    assert!(deleting.finalizers().iter().any(|f| f == "wafl.opt.td/app"));
    assert!(
        placements(&c1.endpoint)
            .await
            .iter()
            .filter(|p| p.app.id == grpc_id)
            .all(|p| !p.withdrawn)
    );
    upload.send_data(data.clone(), true).unwrap();
    assert_eq!(body.data().await.unwrap().unwrap(), data);
    body.flow_control().release_capacity(data.len()).unwrap();
    assert!(body.data().await.is_none());
    assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "0");
    driver_h2.abort();
    gone(&api, "grpc").await;
    // Delete/recreate of the same Kubernetes name is a new WAFL identity.
    api.delete("hello", &DeleteParams::default()).await.unwrap();
    gone(&api, "hello").await;
    let mut zero = desired.clone();
    zero["spec"]["scaling"] = json!({"min":0,"max":1,"idleMs":60000});
    let recreated = api
        .create(
            &PostParams::default(),
            &serde_json::from_value(zero).unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(recreated.uid(), original.uid());
    let cold = observed(&api, "hello", |o| {
        o.data["status"]["desired"] == 0 && o.data["status"]["app"].is_string()
    })
    .await;
    assert_ne!(cold.data["status"]["app"], ready.data["status"]["app"]);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let response = http
        .get(&address)
        .header("host", "hello.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "kubernetes:1");
    observed(&api, "hello", |o| o.data["status"]["ready"] == 1).await;
    api.delete("hello", &DeleteParams::default()).await.unwrap();
    gone(&api, "hello").await;
    assert!(placements(&c1.endpoint).await.is_empty());
    proxy.kill().await.unwrap();
    proxy.wait().await.unwrap();
    c1.stop().await;
    c2.stop().await;
    runner.stop().await;
    for name in [&ns, &other] {
        namespaces
            .delete(name, &DeleteParams::default())
            .await
            .unwrap();
    }
    sql.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    driver.abort();
}
