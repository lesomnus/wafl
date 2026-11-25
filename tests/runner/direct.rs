use super::lifecycle::{call, conditions};
use super::*;
use serde_json::json;
use wafl_runner::{config::Direct, runner::Condition};

fn direct_config(root: &std::path::Path, ports: u16, external: bool) -> Config {
    let probe = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
    let first = probe.local_addr().unwrap().port();
    let advertise = if external {
        // Route lookup only: UDP connect does not send a packet. Exercise a
        // non-loopback peer, as a gateway in another Pod would use.
        let udp = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
        udp.connect("192.0.2.1:9").unwrap();
        let std::net::IpAddr::V4(ip) = udp.local_addr().unwrap().ip() else {
            panic!("IPv4")
        };
        ip
    } else {
        std::net::Ipv4Addr::LOCALHOST
    };
    Config {
        directory: Some(root.join("cache")),
        direct: Some(Direct {
            listen: std::net::Ipv4Addr::UNSPECIFIED,
            advertise,
            directory: root.join("endpoints"),
            first,
            last: first.checked_add(ports - 1).unwrap(),
        }),
        ..config()
    }
}

#[tokio::test]
async fn direct_configuration_keeps_advertisement_and_state_separate_from_cache() {
    let root = tempfile::tempdir().unwrap();
    let good = direct_config(root.path(), 2, false);
    assert!(good.validate().is_ok());
    let mut bad = good.clone();
    bad.direct.as_mut().unwrap().advertise = std::net::Ipv4Addr::UNSPECIFIED;
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.control.set_port(bad.direct.as_ref().unwrap().first);
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.direct.as_mut().unwrap().directory = good.directory.as_ref().unwrap().join("state");
    assert!(bad.validate().is_err());
    // The canonical check must also catch paths that only become nested after
    // resolving '..', rather than relying on their spelling in configuration.
    let mut bad = good;
    bad.direct.as_mut().unwrap().directory = root.path().join("other/../cache/state");
    assert!(bad.validate().is_ok());
    assert!(Runner::new(&bad).await.is_err());
}

async fn add(runner: &Runner, root: &std::path::Path, name: &str, artifact: &str) -> Instance {
    let mut value = app(name, artifact);
    value.idle_ms = 0;
    if artifact == "grpc" {
        value.protocol = "h2c".into();
    }
    let path = root.join(format!("{name}.tar.gz"));
    std::fs::write(&path, value.bundle()).unwrap();
    runner
        .add(value.request(reqwest::Url::from_file_path(path).unwrap().to_string()))
        .await
        .unwrap()
}
async fn response(instance: &Instance, path: &str) -> hyper::Response<hyper::body::Incoming> {
    let socket = TcpStream::connect(&instance.endpoint.address)
        .await
        .unwrap();
    let (mut client, driver) = hyper::client::conn::http1::handshake(TokioIo::new(socket))
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = driver.await;
    });
    client
        .send_request(
            http::Request::builder()
                .uri(path)
                .header("Host", "same.example")
                .header("Connection", "close")
                .body(Empty::<Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap()
}
async fn text(instance: &Instance, path: &str) -> Bytes {
    let response = response(instance, path).await;
    assert_eq!(response.status(), 200);
    response.into_body().collect().await.unwrap().to_bytes()
}
fn condition(instance: &Instance) -> Condition {
    Condition {
        session: Some(instance.session.clone().try_into().unwrap()),
        revision: Some(instance.revision),
    }
}
async fn release(runner: &Runner, instance: &Instance) -> Instance {
    let execution = runner.instance(&instance.id).unwrap();
    execution
        .drain(0, false, Condition::default())
        .await
        .unwrap();
    let draining = execution.snapshot();
    execution
        .release(draining.generation, condition(&draining))
        .await
        .unwrap();
    execution.snapshot()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_sockets_accept_nonloopback_without_routing_headers_and_do_not_reap() {
    let root = tempfile::tempdir().unwrap();
    let config = direct_config(root.path(), 16, true);
    let runner = Runner::new(&config).await.unwrap();
    let a = add(&runner, root.path(), "alpha", "socket").await;
    let b = add(&runner, root.path(), "beta", "socket").await;
    assert!(runner.direct() && a.direct && b.direct);
    assert!(runner.status().ingress.is_empty());
    assert!(
        runner
            .status()
            .capabilities
            .iter()
            .any(|c| c == "direct-endpoint-v1")
    );
    assert_ne!(a.endpoint.address, b.endpoint.address);
    assert_eq!(a.manifest.port, b.manifest.port);
    assert_eq!(a.manifest.idle_ms, 0);
    assert_eq!(text(&a, "/headers").await, "preserved");
    assert_eq!(text(&b, "/").await, "beta:1");
    assert_eq!(text(&a, "/denied").await, "denied");
    runner.reap().await;
    assert_eq!(runner.snapshot(&a.id).unwrap().phase, Phase::Running);
    assert_eq!(text(&a, "/local").await, "8080");
    release(&runner, &a).await;
    release(&runner, &b).await;
    runner.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_drain_requires_gateway_ack_and_fences_stale_release_before_port_reuse() {
    let root = tempfile::tempdir().unwrap();
    let runner = Runner::new(&direct_config(root.path(), 1, false))
        .await
        .unwrap();
    let first = add(&runner, root.path(), "drain", "socket").await;
    let app = runner.instance(&first.id).unwrap();
    let reference = json!({"id": STANDARD.encode(&first.id)});
    let request = json!({"ref":reference,"generation":first.generation.to_string()});
    assert!(app.stop(false, false).await.is_err());
    assert!(runner.erase(&first.id).await.is_err());
    assert!(app.drain(1, false, Condition::default()).await.is_err());
    assert!(
        app.release(first.generation, condition(&first))
            .await
            .is_err()
    );
    let response = response(&first, "/slow").await;
    let drained = call(
        &runner,
        "/wafl.InstanceService/Drain",
        json!({"ref":reference}),
        &conditions(&first),
    )
    .await;
    assert_eq!(drained.0, 200, "{}", drained.1);
    assert_eq!(drained.1["phase"], "PHASE_RUNNING");
    assert_eq!(drained.1["draining"], true);
    assert!(runner.start(&app, Condition::default()).await.is_err());
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "drain:1"
    );
    assert_ne!(
        call(
            &runner,
            "/wafl.InstanceService/Release",
            request.clone(),
            &[]
        )
        .await
        .0,
        200
    );
    assert_ne!(
        call(
            &runner,
            "/wafl.InstanceService/Release",
            request.clone(),
            &conditions(&first)
        )
        .await
        .0,
        200
    );
    let before = app.snapshot();
    let released = call(
        &runner,
        "/wafl.InstanceService/Release",
        request,
        &conditions(&before),
    )
    .await;
    assert_eq!(released.0, 200, "{}", released.1);
    assert_eq!(app.snapshot().phase, Phase::Stopped);
    assert!(app.snapshot().endpoint.as_option().is_none());
    assert!(TcpStream::connect(&first.endpoint.address).await.is_err());
    drop(runner.start(&app, Condition::default()).await.unwrap());
    let second = app.snapshot();
    assert_eq!(
        second.endpoint.address, first.endpoint.address,
        "single port reused only after acknowledgement"
    );
    assert!(second.generation > first.generation);
    app.drain(0, false, Condition::default()).await.unwrap();
    let current = app.snapshot();
    assert!(
        app.release(first.generation, condition(&current))
            .await
            .is_err()
    );
    assert_eq!(
        text(&second, "/").await,
        "drain:1",
        "Drain must not cut live sockets before gateway acknowledgement"
    );
    app.release(current.generation, condition(&current))
        .await
        .unwrap();
    runner.erase(&first.id).await.unwrap();
    let other = add(&runner, root.path(), "other", "socket").await;
    assert_eq!(other.endpoint.address, first.endpoint.address);
    assert_eq!(text(&other, "/").await, "other:1");
    release(&runner, &other).await;
    runner.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_restart_keeps_old_address_reserved_and_skips_busy_ports() {
    let root = tempfile::tempdir().unwrap();
    // A free ephemeral port does not imply that its next two ports are free:
    // parallel tests also open client sockets. Leave spare candidates while
    // still checking the deliberately busy port and the retained old claim.
    let config = direct_config(root.path(), 32, false);
    let occupied =
        std::net::TcpListener::bind(("0.0.0.0", config.direct.as_ref().unwrap().first)).unwrap();
    let old = Runner::new(&config).await.unwrap();
    let first = add(&old, root.path(), "old", "socket").await;
    assert_ne!(
        first.endpoint.address.parse::<SocketAddr>().unwrap().port(),
        occupied.local_addr().unwrap().port()
    );
    old.shutdown().await;
    drop(old);
    let new = Runner::new(&config).await.unwrap();
    let second = add(&new, root.path(), "new", "socket").await;
    assert_ne!(second.session, first.session);
    assert_ne!(second.endpoint.address, first.endpoint.address);
    assert!(TcpStream::connect(&first.endpoint.address).await.is_err());
    assert_eq!(text(&second, "/").await, "new:1");
    let app = new.instance(&second.id).unwrap();
    app.drain(0, false, Condition::default()).await.unwrap();
    let mut wrong = condition(&app.snapshot());
    wrong.session = Some(first.session.try_into().unwrap());
    assert!(app.release(second.generation, wrong).await.is_err());
    release(&new, &second).await;
    new.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_grpc_stream_continues_through_withdrawal() {
    let root = tempfile::tempdir().unwrap();
    let runner = Runner::new(&direct_config(root.path(), 8, false))
        .await
        .unwrap();
    let instance = add(&runner, root.path(), "grpc", "grpc").await;
    assert_eq!(instance.endpoint.protocol, "h2c");
    let socket = TcpStream::connect(&instance.endpoint.address)
        .await
        .unwrap();
    let (mut client, driver) = h2::client::handshake(socket).await.unwrap();
    let driver = tokio::spawn(async move {
        let _ = driver.await;
    });
    let (response, mut send) = client
        .send_request(
            http::Request::builder()
                .uri("http://same.example/echo")
                .header("content-type", "application/grpc")
                .body(())
                .unwrap(),
            false,
        )
        .unwrap();
    let mut body = response.await.unwrap().into_body();
    send.send_data(Bytes::from_static(b"first"), false).unwrap();
    assert_eq!(body.data().await.unwrap().unwrap(), "first");
    let execution = runner.instance(&instance.id).unwrap();
    execution
        .drain(0, false, Condition::default())
        .await
        .unwrap();
    send.send_data(Bytes::from_static(b"second"), true).unwrap();
    assert_eq!(body.data().await.unwrap().unwrap(), "second");
    assert!(body.data().await.is_none());
    assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "0");
    drop(body);
    drop(client);
    drop(send);
    driver.abort();
    let _ = driver.await;
    release(&runner, &instance).await;
    runner.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_contract_rejects_proxy_limits_and_server_fields_and_supports_selection() {
    let root = tempfile::tempdir().unwrap();
    let runner = Runner::new(&direct_config(root.path(), 8, false))
        .await
        .unwrap();
    let instance = add(&runner, root.path(), "select", "socket").await;
    let reference = json!({"id":STANDARD.encode(&instance.id)});
    let (_, selected) = call(
        &runner,
        "/wafl.InstanceService/Get",
        json!({"ref":reference,"select":{"endpoint":true,"direct":true}}),
        &[],
    )
    .await;
    assert_eq!(selected["endpoint"]["address"], instance.endpoint.address);
    assert_eq!(selected["direct"], true);
    let (_, hidden) = call(
        &runner,
        "/wafl.InstanceService/Get",
        json!({"ref":reference,"select":{"name":true}}),
        &[],
    )
    .await;
    assert!(hidden["endpoint"].is_null());
    let mut bad = app("invalid", "socket");
    let path = root.path().join("select.tar.gz");
    let url = reqwest::Url::from_file_path(path).unwrap().to_string();
    assert!(
        runner.add(bad.request(url.clone())).await.is_err(),
        "idle_ms is not honored in direct mode"
    );
    bad.idle_ms = 0;
    bad.max_active = 2;
    assert!(runner.add(bad.request(url.clone())).await.is_err());
    bad.max_active = 0;
    let mut request = bad.request(url);
    request.direct = true;
    assert!(runner.add(request.clone()).await.is_err());
    request.direct = false;
    request.endpoint = Some(Endpoint {
        address: "127.0.0.1:12345".into(),
        ..Default::default()
    })
    .into();
    assert!(runner.add(request).await.is_err());
    release(&runner, &instance).await;
    runner.shutdown().await;
}
