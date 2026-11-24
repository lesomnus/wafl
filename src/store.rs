//! Immutable bundles, cross-process reader pins and demand GC.
use crate::{
    bundle::unpack,
    config::{BundleLimits, Cache},
};
use connectrpc::ConnectError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::OwnedSemaphorePermit,
};

#[derive(Clone)]
pub(crate) struct Store {
    root: PathBuf,
    limits: BundleLimits,
    capacity: u64,
    users: Arc<File>,
}
pub(crate) struct Bundle {
    root: PathBuf,
    pub digest: String,
    lease: File,
    _users: Arc<File>,
}
impl Bundle {
    pub fn main(&self) -> PathBuf {
        self.root.join("main.wasm")
    }
    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }
}
impl Drop for Bundle {
    fn drop(&mut self) {
        let _ = self.lease.set_modified(SystemTime::now());
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    digest: String,
    download_bytes: u64,
    expanded_bytes: u64,
    entries: usize,
    payload_bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    capacity_bytes: u64,
}
fn fault(error: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("bundle store: {error}"))
}
pub(crate) fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|v| {
        v.len() == 64
            && v.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn open_lock(path: &Path) -> Result<File, ConnectError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(fault)
}
fn catalog(root: &Path) -> Result<File, ConnectError> {
    let file = open_lock(&root.join("catalog.lock"))?;
    file.lock().map_err(fault)?;
    Ok(file)
}
impl Store {
    // Call from a blocking worker: locking and GC perform disk IO.
    pub fn new(directory: &Path, limits: BundleLimits, cache: &Cache) -> anyhow::Result<Self> {
        std::fs::create_dir_all(directory)?;
        let root = std::fs::canonicalize(directory)?.join("v2");
        for dir in ["objects/sha256", "refs/sha256", "locks", "tmp", "trash"] {
            std::fs::create_dir_all(root.join(dir))?;
        }
        let _catalog = catalog(&root)?;
        let users = open_lock(&root.join("users.lock"))?;
        let alone = match users.try_lock() {
            Ok(()) => true,
            Err(TryLockError::WouldBlock) => false,
            Err(TryLockError::Error(e)) => return Err(e.into()),
        };
        let policy = root.join("policy.json");
        let previous = if policy.try_exists()? {
            Some(serde_json::from_slice::<Policy>(&std::fs::read(&policy)?)?.capacity_bytes)
        } else {
            None
        };
        anyhow::ensure!(
            alone || previous == Some(cache.capacity_bytes),
            "all live runners sharing a directory must use the same cache.capacity_bytes"
        );
        let store = Self {
            root,
            limits,
            capacity: cache.capacity_bytes,
            users: Arc::new(users),
        };
        if previous != Some(cache.capacity_bytes) {
            // Capacity reduction is the only startup-triggered eviction.
            store.make_room(0)?;
            let mut temp = tempfile::NamedTempFile::new_in(&store.root)?;
            serde_json::to_writer(
                &mut temp,
                &Policy {
                    capacity_bytes: cache.capacity_bytes,
                },
            )?;
            temp.as_file().sync_all()?;
            temp.persist(policy)?;
        }
        if alone {
            store.users.unlock()?;
        }
        store.users.lock_shared()?;
        Ok(store)
    }
    // Catalog protects all pin acquisitions, references and publications.
    fn object(&self, digest: &str) -> Result<Option<Bundle>, ConnectError> {
        if !valid_digest(digest) {
            return Err(fault("invalid cached digest"));
        }
        let root = self.root.join("objects/sha256").join(&digest[7..]);
        if !root.try_exists().map_err(fault)? {
            return Ok(None);
        }
        let r = read_receipt(&root.join("ready.json"))?;
        if r.version != 2
            || r.digest != digest
            || !root.join("data").is_dir()
            || !root.join("main.wasm").is_file()
            || root.join("main.wasm").metadata().map_err(fault)?.len() == 0
        {
            return Err(fault("incomplete or incompatible cached object"));
        }
        if r.download_bytes > self.limits.download_bytes
            || r.expanded_bytes > self.limits.expanded_bytes
            || r.entries > self.limits.entries
        {
            return Err(ConnectError::resource_exhausted(
                "cached bundle exceeds this runner's bundle limits",
            ));
        }
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join("lease"))
            .map_err(fault)?;
        lease.lock_shared().map_err(fault)?;
        lease.set_modified(SystemTime::now()).map_err(fault)?;
        Ok(Some(Bundle {
            root,
            digest: digest.into(),
            lease,
            _users: self.users.clone(),
        }))
    }
    async fn lookup(&self, expected: String, key: String) -> Result<Option<Bundle>, ConnectError> {
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let _catalog = catalog(&store.root)?;
            if !expected.is_empty() {
                return store.object(&expected);
            }
            let reference = store.root.join("refs/sha256").join(format!("{key}.json"));
            if !reference.try_exists().map_err(fault)? {
                return Ok(None);
            }
            let r = read_receipt(&reference)?;
            if r.version != 2 {
                return Err(fault("unsupported source reference version"));
            }
            store.object(&r.digest)
        })
        .await
        .map_err(fault)?
    }
    pub async fn load(
        &self,
        client: &reqwest::Client,
        url: reqwest::Url,
        expected: &str,
        slot: Arc<OwnedSemaphorePermit>,
    ) -> Result<Bundle, ConnectError> {
        let key = hash(url.as_str().as_bytes());
        if let Some(bundle) = self.lookup(expected.into(), key.clone()).await? {
            return Ok(bundle);
        }
        // Fixed stripes bound lock-file metadata across arbitrary source URLs.
        let stripe = &key[..2];
        let source_lock =
            Arc::new(lock(&self.root.join("locks").join(format!("source-{stripe}"))).await?);
        if let Some(bundle) = self.lookup(expected.into(), key.clone()).await? {
            return Ok(bundle);
        }
        let cleanup = self.clone();
        let cleanup_lock = source_lock.clone();
        let cleanup_stripe = stripe.to_owned();
        tokio::task::spawn_blocking(move || {
            let _lock = cleanup_lock;
            cleanup.staging(&cleanup_stripe)
        })
        .await
        .map_err(fault)??;
        let prefix = format!("{key}-");
        let stage = tempfile::Builder::new()
            .prefix(&prefix)
            .tempdir_in(self.root.join("tmp"))
            .map_err(fault)?;
        let (digest, download_bytes) = fetch(
            client,
            url,
            &stage.path().join("archive.gz"),
            self.limits.download_bytes,
        )
        .await?;
        if !expected.is_empty() && expected != digest {
            return Err(ConnectError::invalid_argument(
                "bundle digest does not match downloaded bytes",
            ));
        }
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let _source = source_lock;
            let _slot = slot;
            let stage = unpack(stage, store.limits.expanded_bytes, store.limits.entries)
                .map_err(|e| ConnectError::invalid_argument(format!("invalid bundle: {e:#}")))?;
            let receipt = Receipt {
                version: 2,
                digest: digest.clone(),
                download_bytes,
                expanded_bytes: stage.expanded_bytes,
                entries: stage.entries,
                payload_bytes: stage.payload_bytes,
            };
            let bytes = serde_json::to_vec(&receipt).map_err(fault)?;
            let _catalog = catalog(&store.root)?;
            let bundle = match store.object(&digest)? {
                Some(bundle) => bundle,
                None => {
                    store.make_room(receipt.payload_bytes)?;
                    let mut file =
                        File::create(stage.root.path().join("ready.json")).map_err(fault)?;
                    file.write_all(&bytes).map_err(fault)?;
                    file.sync_all().map_err(fault)?;
                    File::create_new(stage.root.path().join("lease")).map_err(fault)?;
                    std::fs::rename(
                        stage.root.path(),
                        store.root.join("objects/sha256").join(&digest[7..]),
                    )
                    .map_err(fault)?;
                    store
                        .object(&digest)?
                        .ok_or_else(|| fault("publication disappeared"))?
                }
            };
            let mut reference = tempfile::Builder::new()
                .prefix(&prefix)
                .tempfile_in(store.root.join("tmp"))
                .map_err(fault)?;
            reference.write_all(&bytes).map_err(fault)?;
            reference.as_file().sync_all().map_err(fault)?;
            reference
                .persist(store.root.join("refs/sha256").join(format!("{key}.json")))
                .map_err(fault)?;
            Ok(bundle)
        })
        .await
        .map_err(fault)?
    }
    fn staging(&self, held_stripe: &str) -> Result<(), ConnectError> {
        for entry in std::fs::read_dir(self.root.join("tmp")).map_err(fault)? {
            let entry = entry.map_err(fault)?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some(stripe) = name
                .get(..2)
                .filter(|s| s.bytes().all(|b| b.is_ascii_hexdigit()))
            else {
                continue;
            };
            let other = if stripe == held_stripe {
                None
            } else {
                let file = open_lock(&self.root.join("locks").join(format!("source-{stripe}")))?;
                match file.try_lock() {
                    Ok(()) => Some(file),
                    Err(TryLockError::WouldBlock) => continue,
                    Err(TryLockError::Error(e)) => return Err(fault(e)),
                }
            };
            remove(&entry.path())?;
            drop(other);
        }
        Ok(())
    }
    fn make_room(&self, incoming: u64) -> Result<(), ConnectError> {
        if incoming > self.capacity {
            return Err(ConnectError::resource_exhausted(
                "bundle payload exceeds cache.capacity_bytes",
            ));
        }
        // Rename-before-delete leaves disposable trash after a process crash.
        for entry in std::fs::read_dir(self.root.join("trash")).map_err(fault)? {
            remove(&entry.map_err(fault)?.path())?;
        }
        let mut total = 0u64;
        let mut candidates = Vec::new();
        for entry in std::fs::read_dir(self.root.join("objects/sha256")).map_err(fault)? {
            let root = entry.map_err(fault)?.path();
            let r = read_receipt(&root.join("ready.json"))?;
            if r.version != 2
                || !valid_digest(&r.digest)
                || root.file_name().and_then(|s| s.to_str()) != Some(&r.digest[7..])
            {
                return Err(fault("invalid object receipt during GC"));
            }
            total = total
                .checked_add(r.payload_bytes)
                .ok_or_else(|| fault("cache size overflow"))?;
            let lease = OpenOptions::new()
                .read(true)
                .write(true)
                .open(root.join("lease"))
                .map_err(fault)?;
            match lease.try_lock() {
                Ok(()) => {
                    let used = lease.metadata().map_err(fault)?.modified().map_err(fault)?;
                    candidates.push((used, r.digest, r.payload_bytes, root, lease));
                }
                Err(TryLockError::WouldBlock) => {}
                Err(TryLockError::Error(e)) => return Err(fault(e)),
            }
        }
        let needed = total.saturating_sub(self.capacity - incoming);
        if needed == 0 {
            return Ok(());
        }
        if candidates.iter().map(|c| c.2).sum::<u64>() < needed {
            return Err(ConnectError::resource_exhausted(
                "cache is full: referenced bundles cannot be evicted",
            ));
        }
        candidates.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let mut freed = 0;
        for (_, digest, bytes, root, _lease) in candidates {
            let trash = self.root.join("trash").join(&digest[7..]);
            std::fs::rename(root, &trash).map_err(fault)?;
            remove(&trash)?;
            freed += bytes;
            if freed >= needed {
                break;
            }
        }
        for entry in std::fs::read_dir(self.root.join("refs/sha256")).map_err(fault)? {
            let path = entry.map_err(fault)?.path();
            let r = read_receipt(&path)?;
            if !valid_digest(&r.digest) {
                return Err(fault("invalid source reference"));
            }
            if !self
                .root
                .join("objects/sha256")
                .join(&r.digest[7..])
                .try_exists()
                .map_err(fault)?
            {
                std::fs::remove_file(path).map_err(fault)?;
            }
        }
        Ok(())
    }
}
fn remove(path: &Path) -> Result<(), ConnectError> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(fault(e)),
    }
}
fn read_receipt(path: &Path) -> Result<Receipt, ConnectError> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(fault)?
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(fault)?;
    if bytes.len() > 4096 {
        return Err(fault("oversized cache receipt"));
    }
    serde_json::from_slice(&bytes).map_err(fault)
}
async fn lock(path: &Path) -> Result<File, ConnectError> {
    let file = open_lock(path)?;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => tokio::time::sleep(Duration::from_millis(25)).await,
            Err(TryLockError::Error(e)) => return Err(fault(e)),
        }
    }
}

// Source adapters produce the same compressed bytes. Extraction and runtime
// mounting do not depend on how the bytes arrived.
async fn fetch(
    client: &reqwest::Client,
    url: reqwest::Url,
    destination: &Path,
    limit: u64,
) -> Result<(String, u64), ConnectError> {
    let mut output = tokio::fs::File::create(destination).await.map_err(fault)?;
    let mut digest = Sha256::new();
    let mut size = 0u64;
    if url.scheme() == "file" {
        let path = url
            .to_file_path()
            .map_err(|_| ConnectError::invalid_argument("invalid local file URL"))?;
        if !tokio::fs::metadata(&path)
            .await
            .map_err(|_| ConnectError::unavailable("cannot inspect local bundle"))?
            .is_file()
        {
            return Err(ConnectError::invalid_argument(
                "bundle must be a regular file",
            ));
        }
        let mut source = tokio::fs::File::open(path)
            .await
            .map_err(|_| ConnectError::unavailable("cannot open local bundle"))?;
        if !source.metadata().await.map_err(fault)?.is_file() {
            return Err(ConnectError::invalid_argument(
                "bundle must be a regular file",
            ));
        }
        let mut buffer = [0u8; 65536];
        loop {
            let count = source.read(&mut buffer).await.map_err(fault)?;
            if count == 0 {
                break;
            }
            size += count as u64;
            check_size(size, limit)?;
            digest.update(&buffer[..count]);
            output.write_all(&buffer[..count]).await.map_err(fault)?;
        }
    } else {
        let mut response = client
            .get(url)
            .header("accept-encoding", "identity")
            .send()
            .await
            .map_err(|e| {
                ConnectError::unavailable(format!("bundle download: {}", e.without_url()))
            })?;
        if !response.status().is_success() {
            return Err(ConnectError::unavailable(format!(
                "bundle download returned HTTP {}",
                response.status().as_u16()
            )));
        }
        check_size(response.content_length().unwrap_or_default(), limit)?;
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            ConnectError::unavailable(format!("bundle download: {}", e.without_url()))
        })? {
            size += chunk.len() as u64;
            check_size(size, limit)?;
            digest.update(&chunk);
            output.write_all(&chunk).await.map_err(fault)?;
        }
    }
    output.flush().await.map_err(fault)?;
    Ok((format!("sha256:{:x}", digest.finalize()), size))
}
fn check_size(size: u64, limit: u64) -> Result<(), ConnectError> {
    if size > limit {
        Err(ConnectError::resource_exhausted(format!(
            "bundle exceeds {limit} byte download limit"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use connectrpc::ErrorCode;
    use tokio::sync::Semaphore;

    fn source(dir: &Path, name: &str) -> reqwest::Url {
        let path = dir.join(format!("{name}.tgz"));
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (name, data) in [
            ("main.wasm", b"wasm".as_slice()),
            ("data/value", name.as_bytes()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, name, data).unwrap();
        }
        std::fs::write(&path, tar.into_inner().unwrap().finish().unwrap()).unwrap();
        reqwest::Url::from_file_path(path).unwrap()
    }
    async fn load(store: &Store, url: reqwest::Url) -> Result<Bundle, ConnectError> {
        let slot = Arc::new(Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap());
        store.load(&reqwest::Client::new(), url, "", slot).await
    }
    fn store(dir: &Path, capacity: u64) -> Store {
        Store::new(
            dir,
            BundleLimits::default(),
            &Cache {
                capacity_bytes: capacity,
            },
        )
        .unwrap()
    }
    fn objects(store: &Store) -> usize {
        std::fs::read_dir(store.root.join("objects/sha256"))
            .unwrap()
            .count()
    }

    #[tokio::test]
    async fn pinned_objects_deduplicate_and_lru_reclaims_only_unused_payload() {
        let directory = tempfile::tempdir().unwrap();
        let sources = tempfile::tempdir().unwrap();
        let store = store(directory.path(), 10);
        let a = source(sources.path(), "a");
        let b = source(sources.path(), "b");
        let c = source(sources.path(), "c");
        let first = load(&store, a.clone()).await.unwrap();
        let duplicate = load(&store, a.clone()).await.unwrap();
        assert_eq!(objects(&store), 1);
        let second = load(&store, b.clone()).await.unwrap();
        assert_eq!(
            load(&store, c.clone()).await.err().unwrap().code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(
            objects(&store),
            2,
            "capacity failure must not partially evict"
        );
        let first_path = first.root.clone();
        drop(first);
        drop(duplicate);
        let third = load(&store, c.clone()).await.unwrap();
        assert!(!first_path.exists());
        assert!(second.data().join("value").exists());
        // Explicit timestamps keep the LRU assertion independent of clock precision.
        let second_root = second.root.clone();
        drop(second);
        drop(third);
        File::open(second_root.join("lease"))
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH)
            .unwrap();
        drop(load(&store, a).await.unwrap());
        assert!(!second_root.exists());
        assert_eq!(objects(&store), 2);
    }

    #[tokio::test]
    async fn concurrent_publications_and_stricter_cache_readers_obey_limits() {
        let directory = tempfile::tempdir().unwrap();
        let sources = tempfile::tempdir().unwrap();
        let first = store(directory.path(), 5);
        let second = store(directory.path(), 5);
        let a = source(sources.path(), "a");
        let b = source(sources.path(), "b");
        let (one, two) = tokio::join!(load(&first, a.clone()), load(&second, b.clone()));
        assert_eq!(usize::from(one.is_ok()) + usize::from(two.is_ok()), 1);
        assert_eq!(objects(&first), 1);
        let winner = if one.is_ok() { a } else { b };
        for limits in [
            BundleLimits {
                download_bytes: 1,
                ..Default::default()
            },
            BundleLimits {
                expanded_bytes: 1,
                ..Default::default()
            },
            BundleLimits {
                entries: 1,
                ..Default::default()
            },
        ] {
            let strict =
                Store::new(directory.path(), limits, &Cache { capacity_bytes: 5 }).unwrap();
            assert_eq!(
                load(&strict, winner.clone()).await.err().unwrap().code,
                ErrorCode::ResourceExhausted
            );
        }
        let separate = tempfile::tempdir().unwrap();
        let strict = Store::new(
            separate.path(),
            BundleLimits {
                download_bytes: 1,
                ..Default::default()
            },
            &Cache { capacity_bytes: 5 },
        )
        .unwrap();
        assert_eq!(
            load(&strict, winner).await.err().unwrap().code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(objects(&strict), 0);
    }

    #[tokio::test]
    async fn capacity_changes_require_all_readers_to_close_and_leave_v1_untouched() {
        let directory = tempfile::tempdir().unwrap();
        let sources = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("v1/objects")).unwrap();
        std::fs::write(directory.path().join("v1/objects/legacy"), b"protected").unwrap();
        let first = store(directory.path(), 10);
        let bundle = load(&first, source(sources.path(), "a")).await.unwrap();
        drop(first);
        assert!(
            Store::new(
                directory.path(),
                BundleLimits::default(),
                &Cache { capacity_bytes: 4 }
            )
            .is_err()
        );
        drop(bundle);
        let resized = store(directory.path(), 4);
        assert_eq!(objects(&resized), 0);
        assert_eq!(
            std::fs::read(directory.path().join("v1/objects/legacy")).unwrap(),
            b"protected"
        );
    }
}
