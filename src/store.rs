//! Versioned, immutable bundle objects shared by cooperating runner processes.
use crate::bundle::unpack;
use connectrpc::ConnectError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::OwnedSemaphorePermit,
};

const DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;
const EXPANDED_BYTES: u64 = 256 * 1024 * 1024;
const ENTRIES: usize = 4096;

#[derive(Clone)]
pub(crate) struct Store {
    root: PathBuf,
}
pub(crate) struct Bundle {
    root: PathBuf,
    pub digest: String,
}
impl Bundle {
    pub fn main(&self) -> PathBuf {
        self.root.join("main.wasm")
    }
    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u32,
    digest: String,
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

impl Store {
    pub fn new(directory: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(directory)?;
        let root = std::fs::canonicalize(directory)?.join("v1");
        for dir in ["objects/sha256", "refs/sha256", "locks", "tmp"] {
            std::fs::create_dir_all(root.join(dir))?;
        }
        Ok(Self { root })
    }

    fn object(&self, digest: &str) -> Result<Option<Bundle>, ConnectError> {
        if !valid_digest(digest) {
            return Err(fault("invalid cached digest"));
        }
        let root = self.root.join("objects/sha256").join(&digest[7..]);
        if !root.try_exists().map_err(fault)? {
            return Ok(None);
        }
        let receipt = read_receipt(&root.join("ready.json"))?;
        if receipt.version != 1
            || receipt.digest != digest
            || !root.join("data").is_dir()
            || !root.join("main.wasm").is_file()
            || root.join("main.wasm").metadata().map_err(fault)?.len() == 0
        {
            return Err(fault("incomplete or incompatible cached object"));
        }
        Ok(Some(Bundle {
            root,
            digest: digest.into(),
        }))
    }

    pub async fn load(
        &self,
        client: &reqwest::Client,
        url: reqwest::Url,
        expected: &str,
        slot: Arc<OwnedSemaphorePermit>,
    ) -> Result<Bundle, ConnectError> {
        if !expected.is_empty()
            && let Some(bundle) = self.object(expected)?
        {
            return Ok(bundle);
        }
        let key = hash(url.as_str().as_bytes());
        let source_lock =
            Arc::new(lock(&self.root.join("locks").join(format!("source-{key}"))).await?);
        if !expected.is_empty() {
            if let Some(bundle) = self.object(expected)? {
                return Ok(bundle);
            }
        } else {
            let reference = self.root.join("refs/sha256").join(format!("{key}.json"));
            if reference.try_exists().map_err(fault)? {
                let receipt = read_receipt(&reference)?;
                if receipt.version != 1 {
                    return Err(fault("unsupported source reference version"));
                }
                if let Some(bundle) = self.object(&receipt.digest)? {
                    return Ok(bundle);
                }
            }
        }
        // Only the holder of this source lock may remove its abandoned staging.
        let prefix = format!("{key}-");
        for entry in std::fs::read_dir(self.root.join("tmp")).map_err(fault)? {
            let entry = entry.map_err(fault)?;
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                if entry.file_type().map_err(fault)?.is_dir() {
                    std::fs::remove_dir_all(entry.path()).map_err(fault)?;
                } else {
                    std::fs::remove_file(entry.path()).map_err(fault)?;
                }
            }
        }
        let stage = tempfile::Builder::new()
            .prefix(&prefix)
            .tempdir_in(self.root.join("tmp"))
            .map_err(fault)?;
        let digest = fetch(client, url, &stage.path().join("archive.gz")).await?;
        if !expected.is_empty() && expected != digest {
            return Err(ConnectError::invalid_argument(
                "bundle digest does not match downloaded bytes",
            ));
        }
        let extraction_lock = source_lock.clone();
        let extraction_slot = slot.clone();
        let stage = tokio::task::spawn_blocking(move || {
            let _lock = extraction_lock;
            let _slot = extraction_slot;
            unpack(stage, EXPANDED_BYTES, ENTRIES)
        })
        .await
        .map_err(fault)?
        .map_err(|e| ConnectError::invalid_argument(format!("invalid bundle: {e:#}")))?;
        let object_lock = lock(
            &self
                .root
                .join("locks")
                .join(format!("object-{}", &digest[7..])),
        )
        .await?;
        let store = self.clone();
        tokio::task::spawn_blocking(move || {
            let _source_lock = source_lock;
            let _object_lock = object_lock;
            let _slot = slot;
            let receipt = Receipt {
                version: 1,
                digest: digest.clone(),
            };
            let bytes = serde_json::to_vec(&receipt).map_err(fault)?;
            let bundle = match store.object(&digest)? {
                Some(bundle) => bundle,
                None => {
                    let mut file = File::create(stage.path().join("ready.json")).map_err(fault)?;
                    file.write_all(&bytes).map_err(fault)?;
                    file.sync_all().map_err(fault)?;
                    let root = store.root.join("objects/sha256").join(&digest[7..]);
                    // Staging is on this filesystem; publication is one rename.
                    std::fs::rename(stage.path(), &root).map_err(fault)?;
                    Bundle { root, digest }
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
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(fault)?;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => tokio::time::sleep(Duration::from_millis(25)).await,
            Err(TryLockError::Error(error)) => return Err(fault(error)),
        }
    }
}

// Source adapters produce the same compressed bytes. Extraction and runtime
// mounting do not depend on how the bytes arrived.
async fn fetch(
    client: &reqwest::Client,
    url: reqwest::Url,
    destination: &Path,
) -> Result<String, ConnectError> {
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
            check_size(size)?;
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
        check_size(response.content_length().unwrap_or_default())?;
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            ConnectError::unavailable(format!("bundle download: {}", e.without_url()))
        })? {
            size += chunk.len() as u64;
            check_size(size)?;
            digest.update(&chunk);
            output.write_all(&chunk).await.map_err(fault)?;
        }
    }
    output.flush().await.map_err(fault)?;
    Ok(format!("sha256:{:x}", digest.finalize()))
}
fn check_size(size: u64) -> Result<(), ConnectError> {
    if size > DOWNLOAD_BYTES {
        Err(ConnectError::resource_exhausted(
            "bundle exceeds 64 MiB download limit",
        ))
    } else {
        Ok(())
    }
}
