use super::lifecycle::call;
use super::*;
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn seal_fences_delayed_add_and_survives_erase_with_bounded_capacity() {
    let root = tempfile::tempdir().unwrap();
    let mut cfg = config();
    cfg.directory = Some(root.path().join("cache"));
    cfg.capacity.tombstones = 2;
    let runner = Runner::new(&cfg).await.unwrap();
    let a = app("sealed", "socket");
    let key = a.id;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/bundle", listener.local_addr().unwrap());
    let (seen, started) = oneshot::channel();
    let (release, ready) = oneshot::channel();
    let bytes = a.bundle();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut headers = [0; 4096];
        assert!(socket.read(&mut headers).await.unwrap() > 0);
        seen.send(()).unwrap();
        ready.await.unwrap();
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            bytes.len()
        );
        socket.write_all(head.as_bytes()).await.unwrap();
        socket.write_all(&bytes).await.unwrap();
    });
    let request = a.request(url);
    let worker = runner.clone();
    let delayed = request.clone();
    let adding = tokio::spawn(async move { worker.add(delayed).await });
    started.await.unwrap();
    let session = paydirt::format_id(&runner.status().id).unwrap();
    let sealed = call(
        &runner,
        "/wafl.InstanceService/Seal",
        json!({"id":STANDARD.encode(key)}),
        &[("wafl-session", session.clone())],
    )
    .await;
    assert_eq!(sealed.0, 200, "{}", sealed.1);
    release.send(()).unwrap();
    server.await.unwrap();
    assert_eq!(
        adding.await.unwrap().unwrap_err().code,
        connectrpc::ErrorCode::FailedPrecondition
    );
    assert!(runner.snapshot(&key).is_none());
    assert_eq!(
        runner.add(request).await.unwrap_err().code,
        connectrpc::ErrorCode::FailedPrecondition
    );
    let b = app("committed", "socket");
    let path = root.path().join("b.tar.gz");
    std::fs::write(&path, b.bundle()).unwrap();
    let request = b.request(reqwest::Url::from_file_path(path).unwrap().to_string());
    let value = runner.add(request.clone()).await.unwrap();
    runner.seal(&value.id).unwrap();
    runner.erase(&value.id).await.unwrap();
    assert_eq!(
        runner.add(request).await.unwrap_err().code,
        connectrpc::ErrorCode::FailedPrecondition
    );
    runner.seal(&key).unwrap();
    assert_eq!(runner.status().usage.tombstones, 2);
    assert_eq!(
        runner.seal(&paydirt::mint(7)).unwrap_err().code,
        connectrpc::ErrorCode::ResourceExhausted
    );
    assert_eq!(
        call(
            &runner,
            "/wafl.InstanceService/Seal",
            json!({"id":STANDARD.encode(key)}),
            &[]
        )
        .await
        .1["code"],
        "invalid_argument"
    );
    assert_eq!(
        call(
            &runner,
            "/wafl.InstanceService/Seal",
            json!({"id":STANDARD.encode(key)}),
            &[(
                "wafl-session",
                paydirt::format_id(&paydirt::mint(8)).unwrap()
            )]
        )
        .await
        .1["code"],
        "failed_precondition"
    );
    runner.shutdown().await;
}
