use super::*;

#[tokio::test]
#[ignore = "requires WAFL_TEST_CONTROLLER_DATABASE_URL"]
async fn generations_ownership_and_deletion_survive_replicas_and_restart() {
    let database = std::env::var("WAFL_TEST_CONTROLLER_DATABASE_URL").unwrap();
    let (sql, driver) = tokio_postgres::connect(&database, tokio_postgres::NoTls)
        .await
        .unwrap();
    let driver = tokio::spawn(driver);
    let schema = format!("source_{}", uuid::Uuid::new_v4().simple());
    sql.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut url = reqwest::Url::parse(&database).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-c search_path={schema}"));
    let config: Config = serde_json::from_value(json!({
        "database":url.as_str().replace('+',"%20"), "runners":["http://127.0.0.1:9999"],
        "traffic":{"control":"http://127.0.0.1:8082","activate":"http://127.0.0.1:8083","listen":"127.0.0.1:8083"}
    })).unwrap();
    let first = Control::open(config.clone()).await.unwrap();
    let second = Control::open(config.clone()).await.unwrap();
    let owner = Owner {
        cluster: "test".into(),
        namespace: "demo".into(),
        name: "hello".into(),
        uid: uuid::Uuid::new_v4().to_string(),
        tenant: paydirt::parse_id(database::TENANT).unwrap().to_vec(),
        generation: 2,
    };
    let spec: Spec = serde_json::from_value(json!({"domain":"hello.example", "replicas":0,
        "manifest":{"bundle":{"url":"https://example.com/hello.tar.gz"}}}))
    .unwrap();
    let (a, b) = tokio::join!(
        state::apply(&first, owner.clone(), spec.clone()),
        state::apply(&second, owner.clone(), spec.clone())
    );
    a.unwrap();
    b.unwrap();
    assert_eq!(
        first
            .db
            .transaction(|tx| Ok(all::<App>(tx, "App")?.len()))
            .await
            .unwrap(),
        1
    );
    let original = state::snapshot(&first, owner.clone()).await.unwrap();
    assert_eq!(original.app.revision, 1);
    assert!(paydirt::has_domain(&owner.id(10), 10));
    let mut stale = owner.clone();
    stale.generation = 1;
    assert!(state::apply(&second, stale, spec.clone()).await.is_err());
    let mut changed = owner.clone();
    changed.generation = 3;
    let mut invalid = spec.clone();
    invalid.domain = "different.example".into();
    assert!(
        state::apply(&first, changed.clone(), invalid)
            .await
            .is_err()
    );
    assert_eq!(
        state::snapshot(&first, owner.clone())
            .await
            .unwrap()
            .generation,
        2
    );
    // Pinning is execution state. Editing a display name must not clear it or
    // roll the deployment merely because the YAML omitted a digest.
    let id = owner.id(10);
    let digest = format!("sha256:{}", "a".repeat(64));
    let expected = digest.clone();
    first
        .db
        .transaction(move |tx| {
            let mut manifest = (*original.app.manifest).clone();
            manifest.bundle.get_or_insert_default().digest = digest;
            call(
                tx,
                "App",
                "Patch",
                json!({"ref":{"id":encode(&id)},"manifest":manifest}),
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let mut rename = spec.clone();
    rename.name = "renamed".into();
    state::apply(&second, changed.clone(), rename)
        .await
        .unwrap();
    let current = state::snapshot(&first, owner.clone()).await.unwrap();
    assert_eq!(current.app.manifest.bundle.digest, expected);
    assert_eq!(current.app.revision, 1);
    let mut moved = changed.clone();
    moved.namespace = "other".into();
    assert!(state::apply(&first, moved, spec.clone()).await.is_err());
    // Removing scaling is explicit, including when replicas were already zero.
    changed.generation = 4;
    let mut scaled = spec.clone();
    scaled.scaling = Some(Scaling {
        max: 1,
        ..Default::default()
    });
    state::apply(&first, changed.clone(), scaled).await.unwrap();
    changed.generation = 5;
    state::apply(&second, changed.clone(), spec.clone())
        .await
        .unwrap();
    assert!(
        state::snapshot(&first, changed.clone())
            .await
            .unwrap()
            .app
            .scaling
            .as_option()
            .is_none()
    );
    assert!(state::release(&second, changed.clone()).await.unwrap());
    drop(first);
    drop(second);
    let restarted = Control::open(config).await.unwrap();
    changed.generation = 99;
    assert!(
        state::apply(&restarted, changed.clone(), spec.clone())
            .await
            .is_err()
    );
    assert!(state::release(&restarted, changed.clone()).await.unwrap());
    assert!(
        restarted
            .db
            .transaction(|tx| Ok(all::<App>(tx, "App")?.is_empty()))
            .await
            .unwrap()
    );
    let mut recreated = owner.clone();
    recreated.uid = uuid::Uuid::new_v4().to_string();
    recreated.generation = 1;
    assert_ne!(recreated.id(10), owner.id(10));
    state::apply(&restarted, recreated.clone(), spec)
        .await
        .unwrap();
    assert!(state::release(&restarted, recreated).await.unwrap());
    drop(restarted);
    sql.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    driver.abort();
}
