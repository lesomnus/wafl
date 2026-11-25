//! Durable, cross-process port claims. A published claim is NEVER reclaimed by
//! Drop, a timer, a dead PID, or bundle GC. Only an external drain acknowledgement
//! may release it. This keeps stale routes from reaching a different instance.
use crate::{
    config::{Direct, RESERVATION_DOMAIN},
    proto::wafl::{Reservation, ReservationRef, ReservationStatus, reservation_ref},
};
use anyhow::{Context, Result, ensure};
use connectrpc::ConnectError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

pub(crate) struct Ports {
    pub config: Direct,
    root: PathBuf,
    session: [u8; 16],
    owner: Arc<File>,
    next: AtomicU32,
}
#[derive(Serialize, Deserialize)]
struct Owner {
    #[serde(default)]
    version: u32,
    token: String,
    session: String,
    instance: String,
    generation: u64,
}
#[derive(Serialize, Deserialize)]
struct Session {
    version: u32,
    session: String,
}
pub(crate) struct Claim {
    pub address: SocketAddr,
    pub bind: SocketAddr,
    file: PathBuf,
    token: String,
    published: AtomicBool,
    released: Mutex<bool>,
    // Keep the session's OS lock alive until all guest/claim references end.
    _owner: Arc<File>,
}

fn fault(error: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("endpoint journal: {error}"))
}
fn catalog(root: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("catalog.lock"))?;
    file.lock()?;
    Ok(file)
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(
        std::fs::symlink_metadata(path)?.file_type().is_file(),
        "not a regular journal file"
    );
    let mut bytes = Vec::new();
    File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 4096, "journal record exceeds 4096 bytes");
    Ok(serde_json::from_slice(&bytes)?)
}
fn owner_record(path: &Path) -> Result<Owner> {
    let value: Owner = read_json(path)?;
    ensure!(
        uuid::Uuid::parse_str(&value.token).is_ok(),
        "invalid allocation token"
    );
    ensure!(
        paydirt::parse_id(&value.session).is_some_and(|id| paydirt::has_domain(&id, 8)),
        "invalid owner session"
    );
    ensure!(
        paydirt::parse_id(&value.instance).is_some_and(|id| paydirt::has_domain(&id, 7)),
        "invalid owner instance"
    );
    ensure!(value.generation > 0, "invalid owner generation");
    Ok(value)
}
fn session_file(root: &Path, session: &str) -> Result<File> {
    let path = root.join("sessions").join(format!("{session}.json"));
    let value: Session = read_json(&path)?;
    ensure!(
        value.version == 1 && value.session == session,
        "invalid session evidence"
    );
    Ok(OpenOptions::new().read(true).write(true).open(path)?)
}
fn identity(address: SocketAddr) -> Vec<u8> {
    let mut hash = Sha256::new();
    hash.update(b"wafl.reservation.v1\0");
    hash.update(address.to_string());
    let mut id = hash.finalize()[..16].to_vec();
    id[6] = (id[6] & 15) | 0x80;
    id[8] = (id[8] & 63) | 0x80;
    id[9] = RESERVATION_DOMAIN;
    id
}

// Called only under catalog lock. Never prune evidence referenced by a claim,
// and stop pruning if any claim cannot be decoded. Session names are not reused.
fn prune_sessions(root: &Path) -> Result<()> {
    let mut referenced = BTreeSet::new();
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let Ok(owner) = owner_record(&path) else {
                return Ok(());
            };
            referenced.insert(owner.session);
        }
    }
    let sessions = root.join("sessions");
    for entry in std::fs::read_dir(&sessions)? {
        let path = entry?.path();
        let Some(name) = path.file_stem().and_then(|n| n.to_str()) else {
            continue;
        };
        if path.extension().is_none_or(|e| e != "json")
            || referenced.contains(name)
            || !paydirt::parse_id(name).is_some_and(|id| paydirt::has_domain(&id, 8))
        {
            continue;
        }
        let Ok(file) = session_file(root, name) else {
            continue;
        };
        match file.try_lock() {
            Ok(()) => std::fs::remove_file(path)?,
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
    }
    File::open(sessions)?.sync_all()?;
    Ok(())
}
impl Ports {
    pub fn new(config: Direct, session: [u8; 16]) -> Result<Arc<Self>> {
        let root = config
            .directory
            .join("v1")
            .join(config.advertise.to_string());
        let missing: Vec<_> = root
            .ancestors()
            .take_while(|p| !p.exists())
            .map(|p| p.to_owned())
            .collect();
        std::fs::create_dir_all(&root)?;
        // Persist the newly created directories before an endpoint is published.
        File::open(&root)?.sync_all()?;
        File::open(root.parent().unwrap())?.sync_all()?;
        File::open(&config.directory)?.sync_all()?;
        for path in missing {
            File::open(path.parent().unwrap())?.sync_all()?;
        }
        let _catalog = catalog(&root)?;
        let sessions = root.join("sessions");
        std::fs::create_dir_all(&sessions)?;
        File::open(&root)?.sync_all()?;
        prune_sessions(&root)?;
        let session_name = paydirt::format_id(&session).unwrap();
        ensure!(paydirt::has_domain(&session, 8), "invalid runner session");
        let mut owner = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(sessions.join(format!("{session_name}.json")))?;
        owner.lock()?;
        serde_json::to_writer(
            &mut owner,
            &Session {
                version: 1,
                session: session_name,
            },
        )?;
        owner.sync_all()?;
        File::open(&sessions)?.sync_all()?;
        Ok(Arc::new(Self {
            config,
            root,
            session,
            owner: Arc::new(owner),
            next: AtomicU32::new(0),
        }))
    }
    pub fn count(&self) -> u32 {
        u32::from(self.config.last) - u32::from(self.config.first) + 1
    }
    pub async fn reserve(
        self: &Arc<Self>,
        instance: [u8; 16],
        generation: u64,
    ) -> Result<Arc<Claim>> {
        let pool = self.clone();
        tokio::task::spawn_blocking(move || pool.reserve_sync(instance, generation)).await?
    }
    fn reserve_sync(&self, instance: [u8; 16], generation: u64) -> Result<Arc<Claim>> {
        let _catalog = catalog(&self.root)?;
        for _ in 0..self.count() {
            let port = (u32::from(self.config.first)
                + self.next.fetch_add(1, Ordering::Relaxed) % self.count())
                as u16;
            let path = self.root.join(format!("{port}.json"));
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            };
            let token = uuid::Uuid::new_v4().to_string();
            // Failed/partial journal writes remain occupied (fail closed).
            let owner = Owner {
                version: 2,
                token: token.clone(),
                session: paydirt::format_id(&self.session).unwrap(),
                instance: paydirt::format_id(&instance).unwrap(),
                generation,
            };
            file.write_all(&serde_json::to_vec(&owner)?)?;
            file.sync_all()?;
            File::open(&self.root)?.sync_all()?;
            return Ok(Arc::new(Claim {
                address: (self.config.advertise, port).into(),
                bind: (self.config.listen, port).into(),
                file: path,
                token,
                published: AtomicBool::new(false),
                released: Mutex::new(false),
                _owner: self.owner.clone(),
            }));
        }
        anyhow::bail!(
            "direct endpoint range exhausted; withdraw and release endpoints, or reconcile orphan reservations"
        )
    }

    pub fn scope(&self) -> String {
        format!(
            "{}:{}-{}",
            self.config.advertise, self.config.first, self.config.last
        )
    }
    fn port(&self, reference: &ReservationRef) -> Result<Option<u16>, ConnectError> {
        match &reference.key {
            Some(reservation_ref::Key::Address(value)) => {
                let address: SocketAddr = value
                    .parse()
                    .map_err(|_| ConnectError::invalid_argument("invalid reservation address"))?;
                if address.ip() != std::net::IpAddr::V4(self.config.advertise)
                    || !(self.config.first..=self.config.last).contains(&address.port())
                {
                    return Err(ConnectError::invalid_argument(
                        "address is outside this runner's direct pool",
                    ));
                }
                Ok(Some(address.port()))
            }
            Some(reservation_ref::Key::Id(id)) if paydirt::has_domain(id, RESERVATION_DOMAIN) => {
                Ok((self.config.first..=self.config.last)
                    .find(|port| identity((self.config.advertise, *port).into()) == *id))
            }
            _ => Err(ConnectError::invalid_argument(
                "reservation ref requires a domain-9 ID or pool address",
            )),
        }
    }
    fn snapshot(&self, port: u16) -> Option<Reservation> {
        let path = self.root.join(format!("{port}.json"));
        if matches!(std::fs::symlink_metadata(&path), Err(e) if e.kind() == std::io::ErrorKind::NotFound)
        {
            return None;
        }
        let address: SocketAddr = (self.config.advertise, port).into();
        let mut value = Reservation {
            id: identity(address),
            address: address.to_string(),
            ..Default::default()
        };
        let owner = match owner_record(&path) {
            Ok(owner) => owner,
            Err(error) => {
                value.status = ReservationStatus::Corrupt.into();
                value.error = format!("invalid reservation journal: {error:#}");
                return Some(value);
            }
        };
        value.token = owner.token.clone();
        value.session = paydirt::parse_id(&owner.session).unwrap().to_vec();
        value.instance = paydirt::parse_id(&owner.instance).unwrap().to_vec();
        value.generation = owner.generation;
        let status = if owner.version != 2 {
            Err(anyhow::anyhow!(
                "legacy or unsupported journal has no trusted owner lock"
            ))
        } else {
            session_file(&self.root, &owner.session).and_then(|file| match file.try_lock() {
                Ok(()) => Ok(ReservationStatus::Orphan),
                Err(TryLockError::WouldBlock) => Ok(ReservationStatus::Active),
                Err(TryLockError::Error(e)) => Err(e.into()),
            })
        };
        match status {
            Ok(status) => value.status = status.into(),
            Err(error) => {
                value.status = ReservationStatus::Unknown.into();
                value.error = format!("owner liveness is unverified: {error:#}");
            }
        }
        Some(value)
    }
    pub async fn get(
        self: &Arc<Self>,
        reference: ReservationRef,
    ) -> Result<Reservation, ConnectError> {
        let pool = self.clone();
        tokio::task::spawn_blocking(move || {
            let port = pool
                .port(&reference)?
                .ok_or_else(|| ConnectError::not_found("reservation not found"))?;
            let _catalog = catalog(&pool.root).map_err(fault)?;
            pool.snapshot(port)
                .ok_or_else(|| ConnectError::not_found("reservation not found"))
        })
        .await
        .map_err(fault)?
    }
    pub async fn list(self: &Arc<Self>) -> Result<Vec<Reservation>, ConnectError> {
        let pool = self.clone();
        tokio::task::spawn_blocking(move || {
            let _catalog = catalog(&pool.root).map_err(fault)?;
            let mut items = Vec::new();
            for entry in std::fs::read_dir(&pool.root).map_err(fault)? {
                let name = entry.map_err(fault)?.file_name();
                let Some(port) = name
                    .to_str()
                    .and_then(|n| n.strip_suffix(".json"))
                    .and_then(|n| n.parse::<u16>().ok())
                else {
                    continue;
                };
                if name != format!("{port}.json").as_str()
                    || !(pool.config.first..=pool.config.last).contains(&port)
                {
                    continue;
                }
                if let Some(value) = pool.snapshot(port) {
                    items.push(value);
                }
            }
            Ok(items)
        })
        .await
        .map_err(fault)?
    }
    pub async fn reclaim(
        self: &Arc<Self>,
        reference: ReservationRef,
        token: String,
        withdrawn: bool,
    ) -> Result<bool, ConnectError> {
        if !withdrawn || uuid::Uuid::parse_str(&token).is_err() {
            return Err(ConnectError::invalid_argument(
                "Release requires the observed token and withdrawn=true after all gateway routes/connections are drained",
            ));
        }
        let pool = self.clone();
        // A blocking supervisor owns the entire transaction, even if the caller
        // cancels. All new allocators/releases serialize on the same catalog.
        tokio::task::spawn_blocking(move || {
            let Some(port) = pool.port(&reference)? else { return Ok(false); };
            let _catalog = catalog(&pool.root).map_err(fault)?;
            let Some(current) = pool.snapshot(port) else {
                File::open(&pool.root).and_then(|d| d.sync_all()).map_err(fault)?;
                return Ok(false);
            };
            if current.token != token {
                return Err(ConnectError::aborted("reservation owner changed or journal is corrupt; do not retry with a newer token"));
            }
            if current.status != ReservationStatus::Orphan {
                return Err(ConnectError::failed_precondition("reservation owner is active or unverified; live sessions use InstanceService.Drain/Release"));
            }
            let path = pool.root.join(format!("{port}.json"));
            let owner = owner_record(&path).map_err(fault)?;
            let guard = session_file(&pool.root, &owner.session).map_err(fault)?;
            guard.try_lock().map_err(|_| ConnectError::failed_precondition("owner session is active or unverified"))?;
            std::fs::remove_file(path).map_err(fault)?;
            File::open(&pool.root).and_then(|d| d.sync_all()).map_err(fault)?;
            Ok(true)
        }).await.map_err(fault)?
    }
}
impl Claim {
    pub fn publish(&self) {
        self.published.store(true, Ordering::Release);
    }
    pub async fn release(self: &Arc<Self>) -> Result<()> {
        let claim = self.clone();
        tokio::task::spawn_blocking(move || claim.release_sync()).await?
    }
    fn release_sync(&self) -> Result<()> {
        let mut released = self.released.lock().unwrap();
        if *released {
            return Ok(());
        }
        let _catalog = catalog(self.file.parent().unwrap())?;
        match std::fs::read(&self.file) {
            Ok(bytes) => {
                let owner: Owner = serde_json::from_slice(&bytes)?;
                ensure!(
                    owner.token == self.token,
                    "endpoint reservation owner changed"
                );
                std::fs::remove_file(&self.file)?;
            }
            // A previous unlink may have succeeded before directory fsync failed.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("read endpoint reservation"),
        }
        File::open(self.file.parent().unwrap())?.sync_all()?;
        *released = true;
        Ok(())
    }
}
impl Drop for Claim {
    fn drop(&mut self) {
        if !self.published.load(Ordering::Acquire) {
            let _ = self.release_sync();
        }
    }
}

#[cfg(test)]
#[path = "endpoint/tests.rs"]
mod recovery_tests;

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn config(root: &std::path::Path) -> Direct {
        Direct {
            listen: "0.0.0.0".parse().unwrap(),
            advertise: "127.0.0.1".parse().unwrap(),
            directory: root.to_owned(),
            first: 21000,
            last: 21015,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn independent_allocators_coordinate_and_published_claims_survive_drop() {
        let root = tempfile::tempdir().unwrap();
        let a = Ports::new(config(root.path()), paydirt::mint(8)).unwrap();
        let b = Ports::new(config(root.path()), paydirt::mint(8)).unwrap();
        let mut tasks = tokio::task::JoinSet::new();
        for i in 0..16 {
            let pool = if i % 2 == 0 { a.clone() } else { b.clone() };
            tasks.spawn(async move { pool.reserve(paydirt::mint(7), 1).await.unwrap() });
        }
        let mut claims = Vec::new();
        let mut ports = std::collections::BTreeSet::new();
        while let Some(result) = tasks.join_next().await {
            let claim = result.unwrap();
            assert!(ports.insert(claim.address.port()));
            claim.publish();
            claims.push(claim);
        }
        assert!(a.reserve(paydirt::mint(7), 1).await.is_err());
        let first = claims.pop().unwrap();
        first.release().await.unwrap();
        let reused = b.reserve(paydirt::mint(7), 2).await.unwrap();
        assert_eq!(reused.address, first.address);
        reused.publish();
        // Repeated release/drop of the old handle cannot unlink its new owner.
        first.release().await.unwrap();
        drop(first);
        assert!(reused.file.exists());
        drop(claims);
        drop(reused);
        drop(a);
        drop(b);
        let restarted = Ports::new(config(root.path()), paydirt::mint(8)).unwrap();
        assert!(restarted.reserve(paydirt::mint(7), 1).await.is_err());
    }

    #[tokio::test]
    async fn unpublished_claims_return_but_corrupt_or_foreign_records_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        let mut config = config(root.path());
        config.last = config.first;
        let pool = Ports::new(config, paydirt::mint(8)).unwrap();
        let claim = pool.reserve(paydirt::mint(7), 1).await.unwrap();
        let path = claim.file.clone();
        drop(claim);
        assert!(!path.exists());
        std::fs::write(&path, b"partial write").unwrap();
        assert!(pool.reserve(paydirt::mint(7), 1).await.is_err());
        std::fs::remove_file(&path).unwrap();
        let claim = pool.reserve(paydirt::mint(7), 1).await.unwrap();
        claim.publish();
        let mut owner: Owner = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        owner.token = "different owner".into();
        std::fs::write(&path, serde_json::to_vec(&owner).unwrap()).unwrap();
        assert!(claim.release().await.is_err());
        assert!(path.exists());
    }
}
