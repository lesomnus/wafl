//! Durable, cross-process port claims. A published claim is NEVER reclaimed by
//! Drop, a timer, a dead PID, or bundle GC. Only an external drain acknowledgement
//! may release it. This keeps stale routes from reaching a different instance.
use crate::config::Direct;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

pub(crate) struct Ports {
    pub config: Direct,
    root: PathBuf,
    session: [u8; 16],
    next: AtomicU32,
}
#[derive(Serialize, Deserialize)]
struct Owner {
    token: String,
    session: String,
    instance: String,
    generation: u64,
}
pub(crate) struct Claim {
    pub address: SocketAddr,
    pub bind: SocketAddr,
    file: PathBuf,
    token: String,
    published: AtomicBool,
    released: Mutex<bool>,
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
        Ok(Arc::new(Self {
            config,
            root,
            session,
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
            }));
        }
        anyhow::bail!(
            "direct endpoint range exhausted; withdraw and release endpoints, or reconcile orphan reservations"
        )
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
        let owner: Owner = serde_json::from_slice(
            &std::fs::read(&self.file).context("read endpoint reservation")?,
        )?;
        ensure!(
            owner.token == self.token,
            "endpoint reservation owner changed"
        );
        std::fs::remove_file(&self.file)?;
        *released = true;
        File::open(self.file.parent().unwrap())?.sync_all()?;
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
mod tests {
    use super::*;
    fn config(root: &std::path::Path) -> Direct {
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
