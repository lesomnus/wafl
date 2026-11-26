use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cli_discovers_session_drains_resumes_and_rejects_pre_restart_requests() {
    let directory = tempfile::tempdir().unwrap();
    let source = tempfile::tempdir().unwrap();
    let path = source.path().join("bundle.tgz");
    std::fs::write(&path, bundle()).unwrap();
    let url = reqwest::Url::from_file_path(path).unwrap();
    let mut server = Server::new(directory.path(), false).await;
    let status: wafl_runner::proto::wafl::Runner = serde_json::from_slice(&success(
        cli(&server.endpoint, &["--json", "runner", "get"]).await,
    ))
    .unwrap();
    assert_eq!(status.ingress, server.ingress.to_string());
    assert!(status.accepting);
    let session = paydirt::format_id(&status.id).unwrap();
    for args in [vec!["runner", "list"], vec!["runner", "get", &session]] {
        let mut flags = vec!["--json"];
        flags.extend(args);
        let row: wafl_runner::proto::wafl::Runner =
            serde_json::from_slice(&success(cli(&server.endpoint, &flags).await)).unwrap();
        assert_eq!(row.id, status.id);
    }
    let id = paydirt::format_id(&paydirt::mint(7)).unwrap();
    let app = instance(
        &server.endpoint,
        &["add", url.as_str(), "--id", &id, "--alias", "worker"],
    )
    .await;
    assert_eq!(app.session, status.id);
    assert_eq!(paydirt::format_id(&app.id).unwrap(), id);
    let old_revision = app.revision.to_string();
    let drained = instance(
        &server.endpoint,
        &["drain", "@worker", "--timeout-ms", "1000"],
    )
    .await;
    assert!(drained.draining);
    assert_eq!(drained.phase, Phase::Stopped);
    assert_eq!(server.get(&app, "/").await.status(), 503);
    assert!(
        !cli(
            &server.endpoint,
            &[
                "--session",
                &session,
                "--revision",
                &old_revision,
                "instance",
                "start",
                "@worker"
            ]
        )
        .await
        .status
        .success()
    );
    instance(&server.endpoint, &["start", "@worker"]).await;
    let response = server.get(&app, "/slow").await;
    #[cfg(unix)]
    {
        assert!(
            Command::new("kill")
                .args(["-TERM", &server.child.id().unwrap().to_string()])
                .status()
                .await
                .unwrap()
                .success()
        );
        assert!(
            response.text().await.is_ok(),
            "shutdown must finish admitted HTTP bodies"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(10), server.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
    }
    #[cfg(not(unix))]
    drop(response);
    server.stop().await;
    let mut restarted = Server::new(directory.path(), false).await;
    assert!(
        !cli(
            &restarted.endpoint,
            &[
                "--session",
                &session,
                "instance",
                "add",
                url.as_str(),
                "--id",
                &id
            ]
        )
        .await
        .status
        .success()
    );
    instance(&restarted.endpoint, &["add", url.as_str(), "--id", &id]).await;
    for args in [
        vec!["runner", "get"],
        vec!["runner", "list"],
        vec!["instance", "get", &id],
        vec!["instance", "list"],
        vec!["instance", "logs", &id],
        vec!["reservation", "get", "127.0.0.1:20000"],
        vec!["reservation", "list"],
    ] {
        let mut flags = vec!["--session", &session];
        flags.extend(args);
        let output = cli(&restarted.endpoint, &flags).await;
        assert!(!output.status.success(), "stale read succeeded: {flags:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .to_lowercase()
                .contains("runner session changed"),
            "stale read did not reject the session: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    restarted.stop().await;
}

#[tokio::test]
async fn ephemeral_listeners_report_their_actual_addresses() {
    let directory = tempfile::tempdir().unwrap();
    let mut child = command()
        .args(["serve", "runner"])
        .arg(directory.path())
        .args(["--listen", "127.0.0.1:0", "--control", "127.0.0.1:0"])
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stderr.take().unwrap()).lines();
    let line = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let line = lines
                .next_line()
                .await
                .unwrap()
                .expect("runner exited before readiness");
            if line.starts_with("wafl-runner ingress=") {
                break line;
            }
        }
    })
    .await
    .unwrap();
    let addresses: Vec<_> = line
        .split_whitespace()
        .skip(1)
        .map(|part| {
            part.split_once('=')
                .unwrap()
                .1
                .parse::<SocketAddr>()
                .unwrap()
        })
        .collect();
    assert_ne!(addresses[0].port(), 0);
    assert_ne!(addresses[1].port(), 0);
    let endpoint = format!("http://{}", addresses[1]);
    let value: wafl_runner::proto::wafl::Runner =
        serde_json::from_slice(&success(cli(&endpoint, &["--json", "runner", "get"]).await))
            .unwrap();
    assert_eq!(value.ingress, addresses[0].to_string());
    assert_eq!(value.control, addresses[1].to_string());
    child.kill().await.unwrap();
}
