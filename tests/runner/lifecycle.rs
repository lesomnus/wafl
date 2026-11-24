use super::*;
use connectrpc::ErrorCode;
use serde_json::json;
use wafl_runner::{config::Capacity, runner::Condition};

async fn until(mut ready: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !ready() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}
async fn call(
    runner: &Runner,
    procedure: &str,
    body: serde_json::Value,
    headers: &[(&str, String)],
) -> (http::StatusCode, serde_json::Value) {
    use http_body_util::Full;
    use tower::ServiceExt;
    let service = connectrpc::ConnectRpcService::new(wafl_runner::control::router(runner.clone()));
    let mut request = http::Request::builder()
        .method("POST")
        .uri(procedure)
        .header("content-type", "application/json")
        .header("connect-protocol-version", "1");
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let response = service
        .oneshot(
            request
                .body(Full::new(Bytes::from(serde_json::to_vec(&body).unwrap())))
                .unwrap(),
        )
        .await
        .unwrap();
    (
        response.status(),
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
}
fn conditions(instance: &Instance) -> [(&'static str, String); 2] {
    [
        (
            "wafl-session",
            paydirt::format_id(&instance.session).unwrap(),
        ),
        ("wafl-revision", instance.revision.to_string()),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_drain_finishes_body_blocks_autowake_and_rejects_stale_commands() {
    let h = Harness::new(vec![app("drain", "socket")]).await;
    let app = h.instance("drain");
    let reference = json!({"id":STANDARD.encode(app.spec.id)});
    let before = app.snapshot();
    let mut client = h.client().await;
    let response = client.send("drain", "/slow").await;
    let drain = {
        let app = app.clone();
        tokio::spawn(async move { app.drain(2000, false, Condition::default()).await })
    };
    until(|| app.snapshot().draining).await;
    let rejected = h.client().await.send("drain", "/").await;
    assert_eq!(rejected.status(), 503);
    assert_eq!(rejected.headers()["wafl-error"], "draining");
    assert!(!drain.is_finished());
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "drain:1"
    );
    drain.await.unwrap().unwrap();
    assert_eq!(app.snapshot().phase, Phase::Stopped);
    assert!(app.snapshot().draining);
    assert_eq!(h.client().await.send("drain", "/").await.status(), 503);
    let stale = call(
        &h.runner,
        "/wafl.InstanceService/Start",
        json!({"ref":reference}),
        &conditions(&before),
    )
    .await;
    assert_eq!(stale.1["code"], "aborted");
    let current = app.snapshot();
    let started = call(
        &h.runner,
        "/wafl.InstanceService/Start",
        json!({"ref":reference}),
        &conditions(&current),
    )
    .await;
    assert_eq!(started.0, 200, "{}", started.1);
    assert!(!app.snapshot().draining);
    assert_eq!(h.client().await.text("drain", "/").await, "drain:1");
    let old = conditions(&app.snapshot());
    assert_eq!(
        call(
            &h.runner,
            "/wafl.InstanceService/Stop",
            json!({"ref":reference}),
            &[(
                "wafl-session",
                paydirt::format_id(&paydirt::mint(8)).unwrap()
            )]
        )
        .await
        .1["code"],
        "failed_precondition"
    );
    assert_eq!(
        call(
            &h.runner,
            "/wafl.InstanceService/Stop",
            json!({"ref":reference}),
            &[("wafl-revision", "1".into())]
        )
        .await
        .1["code"],
        "invalid_argument"
    );
    h.runner.erase(&app.spec.id).await.unwrap();
    let mut recreated = super::app("drain", "socket");
    recreated.id = app.spec.id;
    h.runner
        .add(recreated.request(h._assets.url("drain")))
        .await
        .unwrap();
    assert_eq!(
        call(&h.runner, "/wafl.InstanceService/Erase", reference, &old)
            .await
            .1["code"],
        "aborted"
    );
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelled_drain_continues_and_deadline_can_preserve_or_force_streams() {
    let h = Harness::new(vec![app("drain", "socket")]).await;
    let app = h.instance("drain");
    let mut client = h.client().await;
    let response = client.send("drain", "/slow").await;
    let drain = {
        let app = app.clone();
        tokio::spawn(async move { app.drain(2000, false, Condition::default()).await })
    };
    until(|| app.snapshot().draining).await;
    drain.abort();
    let _ = drain.await;
    response.into_body().collect().await.unwrap();
    until(|| app.snapshot().phase == Phase::Stopped).await;
    drop(h.runner.start(&app, Condition::default()).await.unwrap());
    let response = client.send("drain", "/slow").await;
    assert_eq!(
        app.drain(10, false, Condition::default())
            .await
            .unwrap_err()
            .code,
        ErrorCode::DeadlineExceeded
    );
    assert!(app.snapshot().draining);
    assert_eq!(app.snapshot().phase, Phase::Running);
    response.into_body().collect().await.unwrap();
    app.drain(1000, false, Condition::default()).await.unwrap();
    drop(h.runner.start(&app, Condition::default()).await.unwrap());
    let response = client.send("drain", "/slow").await;
    app.drain(10, true, Condition::default()).await.unwrap();
    assert_eq!(app.snapshot().phase, Phase::Stopped);
    assert!(response.into_body().collect().await.is_err());
    until(|| h.runner.status().usage.requests == 0).await;
    assert_eq!(h.runner.status().usage.memory_bytes, 0);
    drop(client);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn memory_execution_request_limits_and_runner_generated_queries() {
    let capacity = Capacity {
        running: 1,
        memory_bytes: 64 * 1024 * 1024,
        requests: 2,
        ..Default::default()
    };
    let h = Harness::limits(vec![app("one", "socket")], capacity).await;
    let one = h.instance("one");
    let two = app("two", "socket");
    let assets = Assets::new([("two".into(), two.bundle())].into()).await;
    let error = h
        .runner
        .add(two.request(assets.url("two")))
        .await
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceExhausted);
    assert_eq!(h.runner.list().len(), 1);
    assert_eq!(h.runner.status().usage.running, 1);
    assert_eq!(h.runner.status().usage.memory_bytes, 64 * 1024 * 1024);
    let a = h.runner.acquire(&one).await.unwrap();
    let b = h.runner.acquire(&one).await.unwrap();
    assert_eq!(
        h.runner.acquire(&one).await.err().unwrap().code,
        ErrorCode::ResourceExhausted
    );
    drop(a);
    drop(b);
    one.stop(false, false).await.unwrap();
    assert_eq!(h.runner.status().usage.memory_bytes, 0);
    h.runner.add(two.request(assets.url("two"))).await.unwrap();
    assert_eq!(
        h.runner.status().usage.compilations,
        1,
        "different data/ and manifests reuse identical main.wasm"
    );
    let (status, body) = call(&h.runner, "/wafl.RunnerService/List", json!({}), &[]).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["items"][0]["id"], STANDARD.encode(h.runner.session()));
    assert_eq!(body["items"][0]["capacity"]["running"], "1");
    assert_eq!(
        body["items"][0]["usage"]["requests"],
        serde_json::Value::Null
    );
    let selected = call(
        &h.runner,
        "/wafl.RunnerService/Get",
        json!({"ref":{"alias":"self"},"select":{"capacity":true}}),
        &[],
    )
    .await
    .1;
    assert!(!selected["capacity"].is_null());
    assert!(selected["runtime"].is_null());
    assert_eq!(
        call(
            &h.runner,
            "/wafl.RunnerService/Get",
            json!({"ref":{"id":STANDARD.encode(paydirt::mint(8))}}),
            &[]
        )
        .await
        .1["code"],
        "not_found"
    );
    let wrong_session = [(
        "wafl-session",
        paydirt::format_id(&paydirt::mint(8)).unwrap(),
    )];
    assert_eq!(
        call(
            &h.runner,
            "/wafl.InstanceService/Add",
            serde_json::to_value(two.request(assets.url("two"))).unwrap(),
            &wrong_session
        )
        .await
        .1["code"],
        "failed_precondition"
    );
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn startup_queue_is_common_to_start_and_autowake_and_releases_reservations() {
    let capacity = Capacity {
        startups: 1,
        waiting: 1,
        queue_ms: 500,
        ..Default::default()
    };
    let mut apps = Vec::new();
    for name in ["a", "b", "c"] {
        let mut spec = app(name, "socket");
        spec.env.insert("MODE".into(), "delay".into());
        spec.env.insert("DELAY_MS".into(), "1000".into());
        apps.push(spec);
    }
    let h = Harness::limits(apps, capacity).await;
    for name in ["a", "b", "c"] {
        h.instance(name).stop(false, false).await.unwrap();
    }
    let first = {
        let runner = h.runner.clone();
        let app = h.instance("a");
        tokio::spawn(async move { runner.start(&app, Condition::default()).await })
    };
    until(|| h.runner.status().usage.startups == 1).await;
    let second = {
        let runner = h.runner.clone();
        let app = h.instance("b");
        tokio::spawn(async move { runner.acquire(&app).await })
    };
    until(|| h.runner.status().usage.waiting == 1).await;
    assert_eq!(
        h.runner.acquire(&h.instance("c")).await.err().unwrap().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        second.await.unwrap().err().unwrap().code,
        ErrorCode::DeadlineExceeded
    );
    drop(first.await.unwrap().unwrap());
    until(|| h.runner.status().usage.requests == 0).await;
    assert_eq!(h.runner.status().usage.waiting, 0);
    assert_eq!(h.runner.status().usage.startups, 0);
    assert_eq!(h.runner.status().usage.running, 1);
    assert_eq!(h.runner.status().usage.memory_bytes, 64 * 1024 * 1024);
    h.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grpc_deadline_includes_cold_start_and_open_stream_lifetime() {
    let mut spec = app("deadline", "grpc");
    spec.protocol = "h2c".into();
    spec.env.insert("MODE".into(), "delay".into());
    let h = Harness::new(vec![spec]).await;
    let app = h.instance("deadline");
    app.stop(false, false).await.unwrap();
    let (mut client, connection) =
        h2::client::handshake(TcpStream::connect(h.address).await.unwrap())
            .await
            .unwrap();
    let driver = tokio::spawn(connection);
    let request = |timeout: &str| {
        http::Request::builder()
            .method("POST")
            .uri("http://grpc.test/echo.Echo/Listen")
            .header(
                proxy::INSTANCE_HEADER,
                paydirt::format_id(&app.spec.id).unwrap(),
            )
            .header("content-type", "application/grpc")
            .header("grpc-timeout", timeout)
            .body(())
            .unwrap()
    };
    let (response, _output) = client.send_request(request("20m"), false).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.headers()["grpc-status"], "4");
    assert_eq!(response.headers()["wafl-error"], "deadline");
    assert_eq!(h.runner.status().usage.requests, 0);
    // Shared startup continues independently; no request body was replayed.
    until(|| app.snapshot().phase == Phase::Running).await;
    app.stop(false, false).await.unwrap();
    let (response, mut output) = client.send_request(request("600m"), false).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), response)
        .await
        .unwrap()
        .unwrap();
    let echoed = response.headers()["echo-timeout"].to_str().unwrap();
    let (number, unit) = echoed.split_at(echoed.len() - 1);
    let nanos = number.parse::<u128>().unwrap()
        * match unit {
            "n" => 1,
            "u" => 1000,
            "m" => 1_000_000,
            _ => panic!("unexpected timeout {echoed}"),
        };
    assert!(
        nanos < 500_000_000,
        "cold start must consume the forwarded timeout: {echoed}"
    );
    let mut body = response.into_body();
    let payload = Bytes::from_static(b"\x00\x00\x00\x00\x05\x0a\x03abc");
    output.send_data(payload.clone(), false).unwrap();
    let data = body.data().await.unwrap().unwrap();
    assert_eq!(data, payload);
    body.flow_control().release_capacity(data.len()).unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(5), body.data())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "4");
    until(|| h.runner.status().usage.requests == 0).await;
    // Do not consume a response larger than the downstream HTTP/2 window.
    // Deadline cleanup must work even when ResponseBody stops being polled.
    let (response, mut blocked) = client.send_request(request("100m"), false).unwrap();
    blocked
        .send_data(Bytes::from(vec![0u8; 2 * 1024 * 1024]), false)
        .unwrap();
    let unread = response.await.unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    until(|| h.runner.status().usage.requests == 0).await;
    drop(unread);
    drop(blocked);
    let (response, _) = client.send_request(request("123456789S"), true).unwrap();
    assert_eq!(response.await.unwrap().status(), 400);
    driver.abort();
    h.finish().await;
}
