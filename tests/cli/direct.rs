use super::*;
use wafl_runner::proto::wafl::{Reservation, ReservationStatus};

async fn reservation(endpoint: &str, reference: &str) -> Reservation {
    serde_json::from_slice(&success(
        cli(endpoint, &["--json", "reservation", "get", reference]).await,
    ))
    .unwrap()
}

async fn release(server: &Server, reference: &str) -> Instance {
    let app = instance(&server.endpoint, &["drain", reference]).await;
    let session = paydirt::format_id(&app.session).unwrap();
    let revision = app.revision.to_string();
    let generation = app.generation.to_string();
    serde_json::from_slice(&success(
        cli(
            &server.endpoint,
            &[
                "--json",
                "--session",
                &session,
                "--revision",
                &revision,
                "instance",
                "release",
                reference,
                "--generation",
                &generation,
            ],
        )
        .await,
    ))
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cli_direct_processes_share_claims_and_crash_never_reuses_old_endpoint() {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let state = directory.path().join("endpoints");
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let first = probe.local_addr().unwrap().port();
    assert!(first < 65528);
    let mut options = serde_json::json!({"direct":{
        "listen":"0.0.0.0", "advertise":"127.0.0.1", "directory":state,
        "first":first, "last":first + 7
    }});
    drop(probe);
    let assets = Assets::new(false).await;
    let mut one = Server::options(&cache, false, Some(options.clone())).await;
    let mut two = Server::options(&cache, false, Some(options.clone())).await;
    let args = ["add", &assets.url, "--alias", "worker"];
    let (a, b) = tokio::join!(
        instance(&one.endpoint, &args),
        instance(&two.endpoint, &args)
    );
    assert!(a.direct && b.direct);
    assert_ne!(a.endpoint.address, b.endpoint.address);
    let observed = reservation(&two.endpoint, &a.endpoint.address).await;
    let slot = paydirt::format_id(&observed.id).unwrap();
    assert_eq!(observed.status, ReservationStatus::Active);
    assert_eq!(observed.instance, a.id);
    assert_eq!(observed.session, a.session);
    assert_eq!(
        reservation(&one.endpoint, &slot).await.token,
        observed.token
    );
    let recover = [
        "reservation",
        "release",
        slot.as_str(),
        "--token",
        observed.token.as_str(),
        "--withdrawn",
    ];
    for endpoint in [&one.endpoint, &two.endpoint] {
        assert!(
            !cli(endpoint, &recover).await.status.success(),
            "live owners cannot be reclaimed by themselves or another process"
        );
    }
    assert!(
        !cli(&two.endpoint, &recover[..5]).await.status.success(),
        "explicit withdrawal confirmation is required"
    );
    let listing = String::from_utf8(success(
        cli(&two.endpoint, &["--json", "reservation", "list"]).await,
    ))
    .unwrap();
    assert_eq!(listing.lines().count(), 2);
    assert_eq!(assets.count.load(Ordering::SeqCst), 1);
    assert!(
        tokio::net::TcpStream::connect(one.ingress).await.is_err(),
        "direct host must not open the runner HTTP ingress"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    for app in [&a, &b] {
        assert_eq!(
            client
                .get(format!("http://{}/data", app.endpoint.address))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
            "shared data"
        );
    }
    let columns = String::from_utf8(success(
        cli(&one.endpoint, &["instance", "get", "@worker"]).await,
    ))
    .unwrap();
    assert_eq!(
        columns.trim().split('\t').collect::<Vec<_>>()[2],
        a.endpoint.address
    );
    let status: wafl_runner::proto::wafl::Runner = serde_json::from_slice(&success(
        cli(&one.endpoint, &["--json", "runner", "get"]).await,
    ))
    .unwrap();
    assert!(status.ingress.is_empty());
    assert!(
        status
            .capabilities
            .iter()
            .any(|c| c == "direct-endpoint-v1")
    );
    assert!(
        status
            .capabilities
            .iter()
            .any(|c| c == "reservation-recovery-v1")
    );
    drop(client); // This test has no gateways; close all direct client pools.
    one.stop().await; // SIGKILL: no process shutdown cleanup can free the claim.
    assert!(
        tokio::net::TcpStream::connect(&a.endpoint.address)
            .await
            .is_err()
    );
    let port: SocketAddr = a.endpoint.address.parse().unwrap();
    assert!(
        state
            .join("v1/127.0.0.1")
            .join(format!("{}.json", port.port()))
            .exists()
    );
    assert_eq!(
        reservation(&two.endpoint, &slot).await.status,
        ReservationStatus::Orphan
    );
    let mut restarted = Server::options(&cache, false, Some(options.clone())).await;
    let c = instance(&restarted.endpoint, &args).await;
    assert_ne!(c.session, a.session);
    assert_ne!(c.endpoint.address, a.endpoint.address);
    assert_ne!(c.endpoint.address, b.endpoint.address);
    assert_eq!(
        reservation(&restarted.endpoint, &slot).await.token,
        observed.token
    );
    // SIGKILL released the owner lock, but did not release the occupied port.
    // The operator has now withdrawn every old route/pool; only this token is acknowledged.
    assert_eq!(
        String::from_utf8(success(cli(&restarted.endpoint, &recover).await))
            .unwrap()
            .trim(),
        "true"
    );
    assert_eq!(
        String::from_utf8(success(cli(&two.endpoint, &recover).await))
            .unwrap()
            .trim(),
        "false"
    );
    options["direct"]["first"] = port.port().into();
    options["direct"]["last"] = port.port().into();
    let mut replacement = Server::options(&cache, false, Some(options)).await;
    let fresh = instance(&replacement.endpoint, &args).await;
    assert_eq!(fresh.endpoint.address, a.endpoint.address);
    let current = reservation(&two.endpoint, &slot).await;
    assert_eq!(observed.id, current.id);
    assert_ne!(observed.token, current.token);
    assert!(
        !cli(&two.endpoint, &recover).await.status.success(),
        "a late retry must not unlink the replacement allocation"
    );
    assert_eq!(
        reqwest::get(format!("http://{}/data", fresh.endpoint.address))
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "shared data"
    );
    assert!(
        !cli(
            &two.endpoint,
            &["instance", "release", "@worker", "--generation", "1"]
        )
        .await
        .status
        .success(),
        "Release must require explicit observed session/revision"
    );
    for server in [&two, &restarted, &replacement] {
        let stopped = release(server, "@worker").await;
        assert_eq!(stopped.phase, Phase::Stopped);
        assert!(stopped.endpoint.as_option().is_none());
    }
    two.stop().await;
    restarted.stop().await;
    replacement.stop().await;
}
