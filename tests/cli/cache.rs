use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stopped_instance_pins_data_across_processes_until_erase_or_crash() {
    let directory = tempfile::tempdir().unwrap();
    let sources = tempfile::tempdir().unwrap();
    let a = bundle_data(b"aaaaaaaaaaa");
    let b = bundle_data(b"bbbbbbbbbbb");
    let c = bundle_data(b"ccccccccccc");
    // Budget fits exactly one decoded bundle, not two compressed archives.
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(a.as_slice()));
    let size: u64 = archive.entries().unwrap().map(|e| e.unwrap().size()).sum();
    let options = Some(serde_json::json!({"cache":{"capacity_bytes":size}}));
    let mut first = Server::options(directory.path(), false, options.clone()).await;
    let mut second = Server::options(directory.path(), false, options).await;
    let mut urls = Vec::new();
    for (name, bytes) in [("a", a), ("b", b), ("c", c)] {
        let path = sources.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        urls.push(reqwest::Url::from_file_path(path).unwrap().to_string());
    }
    let one = instance(&first.endpoint, &["add", &urls[0], "--alias", "a"]).await;
    instance(&first.endpoint, &["stop", "@a"]).await;
    let duplicate = instance(&second.endpoint, &["add", &urls[0], "--alias", "copy"]).await;
    assert_eq!(one.manifest.bundle.digest, duplicate.manifest.bundle.digest);
    let rejected = cli(&second.endpoint, &["instance", "add", &urls[1]]).await;
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("resource_exhausted"));
    success(cli(&second.endpoint, &["instance", "stop", "@copy"]).await);
    success(cli(&second.endpoint, &["instance", "erase", "@copy"]).await);
    assert!(
        !cli(&second.endpoint, &["instance", "add", &urls[1]])
            .await
            .status
            .success()
    );
    success(cli(&first.endpoint, &["instance", "erase", "@a"]).await);
    instance(&second.endpoint, &["add", &urls[1], "--alias", "b"]).await;
    let old = directory
        .path()
        .join("v2/objects/sha256")
        .join(&one.manifest.bundle.digest[7..]);
    assert!(!old.exists());
    second.stop().await;
    // OS releases the other process's pins after SIGKILL.
    instance(&first.endpoint, &["add", &urls[2], "--alias", "c"]).await;
    assert_eq!(
        std::fs::read_dir(directory.path().join("v2/objects/sha256"))
            .unwrap()
            .count(),
        1
    );
    first.stop().await;
}
