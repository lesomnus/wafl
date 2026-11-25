use super::*;
use connectrpc::ErrorCode;
use std::time::Duration;

fn reference(claim: &Claim) -> ReservationRef {
    ReservationRef {
        key: Some(reservation_ref::Key::Address(claim.address.to_string())),
        ..Default::default()
    }
}
fn pool(root: &Path) -> Arc<Ports> {
    let mut config = super::tests::config(root);
    config.last = config.first;
    Ports::new(config, paydirt::mint(8)).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn liveness_guard_outlives_pool_and_stale_release_cannot_remove_new_owner() {
    let root = tempfile::tempdir().unwrap();
    let owner = pool(root.path());
    let claim = owner.reserve(paydirt::mint(7), 1).await.unwrap();
    claim.publish();
    let target = reference(&claim);
    let token = claim.token.clone();
    let observer = pool(root.path());
    let initial = observer.get(target.clone()).await.unwrap();
    assert_eq!(initial.status, ReservationStatus::Active);
    drop(owner);
    assert_eq!(
        observer.get(target.clone()).await.unwrap().status,
        ReservationStatus::Active
    );
    assert_eq!(
        observer
            .reclaim(target.clone(), token.clone(), true)
            .await
            .unwrap_err()
            .code,
        ErrorCode::FailedPrecondition
    );
    drop(claim);
    assert_eq!(
        observer.get(target.clone()).await.unwrap().status,
        ReservationStatus::Orphan
    );
    assert_eq!(
        observer
            .reclaim(target.clone(), token.clone(), false)
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        observer
            .reclaim(target.clone(), uuid::Uuid::new_v4().to_string(), true)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Aborted
    );

    let other = pool(root.path());
    let (a, b) = tokio::join!(
        observer.reclaim(target.clone(), token.clone(), true),
        other.reclaim(target.clone(), token.clone(), true)
    );
    assert_ne!(
        a.unwrap(),
        b.unwrap(),
        "exactly one concurrent reclaimer unlinks the reservation"
    );
    let new = other.reserve(paydirt::mint(7), 2).await.unwrap();
    new.publish();
    let current = observer.get(target.clone()).await.unwrap();
    assert_eq!(
        initial.id, current.id,
        "slot ID remains stable across owners"
    );
    assert_ne!(initial.token, current.token);
    assert_eq!(
        observer
            .reclaim(target, token, true)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Aborted
    );
    assert!(new.file.exists());
    new.release().await.unwrap();
}

#[tokio::test]
async fn legacy_corruption_and_missing_session_evidence_are_visible_but_never_reclaimed() {
    let root = tempfile::tempdir().unwrap();
    let owner = pool(root.path());
    let claim = owner.reserve(paydirt::mint(7), 1).await.unwrap();
    claim.publish();
    let target = reference(&claim);
    let token = claim.token.clone();
    let path = claim.file.clone();
    let proof = owner.root.join("sessions").join(format!(
        "{}.json",
        paydirt::format_id(&owner.session).unwrap()
    ));
    let original: serde_json::Value = read_json(&path).unwrap();
    drop((owner, claim));
    let observer = pool(root.path());
    for variant in [
        {
            let mut v = original.clone();
            v.as_object_mut().unwrap().remove("version");
            v
        },
        {
            let mut v = original.clone();
            v["version"] = 99.into();
            v
        },
        original.clone(),
    ] {
        if variant["version"] == 2 {
            std::fs::remove_file(&proof).unwrap();
        }
        std::fs::write(&path, serde_json::to_vec(&variant).unwrap()).unwrap();
        let value = observer.get(target.clone()).await.unwrap();
        assert_eq!(value.status, ReservationStatus::Unknown);
        assert!(!value.error.is_empty());
        assert_eq!(
            observer
                .reclaim(target.clone(), token.clone(), true)
                .await
                .unwrap_err()
                .code,
            ErrorCode::FailedPrecondition
        );
        assert!(path.exists());
    }
    for bytes in [b"{partial".as_slice(), &vec![b'x'; 4097]] {
        std::fs::write(&path, bytes).unwrap();
        let value = observer.get(target.clone()).await.unwrap();
        assert_eq!(value.status, ReservationStatus::Corrupt);
        assert!(
            observer
                .reclaim(target.clone(), token.clone(), true)
                .await
                .is_err()
        );
        assert_eq!(observer.list().await.unwrap().len(), 1);
        assert!(observer.reserve(paydirt::mint(7), 1).await.is_err());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accepted_reclaim_survives_cancellation_while_waiting_for_catalog() {
    let root = tempfile::tempdir().unwrap();
    let owner = pool(root.path());
    let claim = owner.reserve(paydirt::mint(7), 1).await.unwrap();
    claim.publish();
    let target = reference(&claim);
    let token = claim.token.clone();
    let path = claim.file.clone();
    drop((owner, claim));
    let observer = pool(root.path());
    let catalog = catalog(&observer.root).unwrap();
    let worker = observer.clone();
    let task = tokio::spawn(async move { worker.reclaim(target, token, true).await });
    tokio::time::timeout(Duration::from_secs(5), async {
        // The third reference belongs to the blocking transaction supervisor.
        while Arc::strong_count(&observer) < 3 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    drop(catalog);
    tokio::time::timeout(Duration::from_secs(5), async {
        while path.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    assert!(observer.list().await.unwrap().is_empty());
}

#[tokio::test]
async fn startup_prunes_only_unreferenced_dead_session_evidence() {
    let root = tempfile::tempdir().unwrap();
    let vacant = pool(root.path());
    let proof = |pool: &Ports| {
        pool.root.join("sessions").join(format!(
            "{}.json",
            paydirt::format_id(&pool.session).unwrap()
        ))
    };
    let old = proof(&vacant);
    drop(vacant);
    let active = pool(root.path());
    assert!(!old.exists());
    let active_proof = proof(&active);
    let claim = active.reserve(paydirt::mint(7), 1).await.unwrap();
    claim.publish();
    let observer = pool(root.path());
    assert!(
        active_proof.exists(),
        "live session evidence is never removed"
    );
    drop((claim, active, observer));
    let restarted = pool(root.path());
    assert!(
        active_proof.exists(),
        "dead owner's published claim still needs its proof"
    );
    assert_eq!(
        restarted.list().await.unwrap()[0].status,
        ReservationStatus::Orphan
    );
}
