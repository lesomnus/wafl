use anyhow::{Context, Result, ensure};
use connectrpc::ConnectError;
use flate2::read::MultiGzDecoder;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    path::{Component, PathBuf},
    sync::Arc,
};
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, sync::OwnedSemaphorePermit};

pub(crate) const DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;
const EXPANDED_BYTES: u64 = 256 * 1024 * 1024;
const ENTRIES: usize = 4096;

/// A validated, private directory whose lifetime follows the instance.
pub(crate) struct Bundle {
    root: TempDir,
}
impl Bundle {
    pub fn main(&self) -> PathBuf {
        self.root.path().join("main.wasm")
    }
    pub fn data(&self) -> PathBuf {
        self.root.path().join("data")
    }

    pub async fn load(
        client: &reqwest::Client,
        url: reqwest::Url,
        slot: Arc<OwnedSemaphorePermit>,
    ) -> Result<Self, ConnectError> {
        let root = tempfile::Builder::new()
            .prefix("wafl-bundle-")
            .tempdir()
            .map_err(|_| ConnectError::internal("cannot create bundle directory"))?;
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
        if response
            .content_length()
            .is_some_and(|n| n > DOWNLOAD_BYTES)
        {
            return Err(ConnectError::resource_exhausted(
                "bundle exceeds 64 MiB download limit",
            ));
        }
        let mut file = tokio::fs::File::create(root.path().join("archive.gz"))
            .await
            .map_err(|_| ConnectError::internal("cannot stage bundle"))?;
        let mut total = 0u64;
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            ConnectError::unavailable(format!("bundle download: {}", e.without_url()))
        })? {
            total += chunk.len() as u64;
            if total > DOWNLOAD_BYTES {
                return Err(ConnectError::resource_exhausted(
                    "bundle exceeds 64 MiB download limit",
                ));
            }
            file.write_all(&chunk)
                .await
                .map_err(|_| ConnectError::internal("cannot stage bundle"))?;
        }
        file.flush()
            .await
            .map_err(|_| ConnectError::internal("cannot finish bundle download"))?;
        drop(file);
        // The blocking task owns its directory, including if the RPC is cancelled.
        tokio::task::spawn_blocking(move || {
            // Blocking work keeps its capacity even if its caller is cancelled.
            let _slot = slot;
            unpack(root, EXPANDED_BYTES, ENTRIES)
        })
        .await
        .map_err(|_| ConnectError::internal("bundle extraction task failed"))?
        .map_err(|e| ConnectError::invalid_argument(format!("invalid bundle: {e:#}")))
    }
}

fn unpack(root: TempDir, limit: u64, entries: usize) -> Result<Bundle> {
    let decoder = MultiGzDecoder::new(File::open(root.path().join("archive.gz"))?);
    let mut input = decoder.take(limit + 1);
    let mut paths = BTreeSet::new();
    {
        let mut archive = tar::Archive::new(&mut input);
        for (index, entry) in archive.entries()?.enumerate() {
            ensure!(index < entries, "too many archive entries");
            let mut entry = entry.context("read tar entry")?;
            let raw = entry.path()?;
            let raw = raw
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("paths must be UTF-8"))?;
            ensure!(
                raw.len() <= 4096 && !raw.contains(['\\', '\0']),
                "invalid archive path"
            );
            let mut path = PathBuf::new();
            for component in std::path::Path::new(raw).components() {
                match component {
                    Component::CurDir => {}
                    Component::Normal(part) => path.push(part),
                    _ => anyhow::bail!("archive path must stay inside the bundle"),
                }
            }
            let kind = entry.header().entry_type();
            ensure!(
                kind.is_file() || kind.is_dir(),
                "only regular files and directories are allowed"
            );
            if path.as_os_str().is_empty() {
                ensure!(kind.is_dir(), "archive root must be a directory");
                continue;
            }
            ensure!(
                path == std::path::Path::new("main.wasm") || path.starts_with("data"),
                "only main.wasm and data/ are allowed"
            );
            ensure!(paths.insert(path.clone()), "duplicate archive path");
            ensure!(
                path != std::path::Path::new("main.wasm") || kind.is_file(),
                "main.wasm must be a regular file"
            );
            ensure!(
                path != std::path::Path::new("data") || kind.is_dir(),
                "data must be a directory"
            );
            let destination = root.path().join(path);
            if kind.is_dir() {
                ensure!(entry.size() == 0, "directory has a nonempty payload");
                std::fs::create_dir_all(destination)?;
            } else {
                ensure!(entry.size() <= limit, "file exceeds expanded size limit");
                std::fs::create_dir_all(destination.parent().unwrap())?;
                let mut file = File::create_new(destination)?;
                io::copy(&mut entry, &mut file)?;
            }
        }
    }
    // Validate the gzip checksum/footer too. The limit includes tar metadata.
    let mut padding = [0u8; 8192];
    loop {
        let read = input.read(&mut padding)?;
        if read == 0 {
            break;
        }
        ensure!(
            padding[..read].iter().all(|v| *v == 0),
            "unexpected data after tar end marker"
        );
    }
    ensure!(input.limit() != 0, "bundle exceeds expanded size limit");
    let main = root.path().join("main.wasm");
    ensure!(
        main.is_file() && main.metadata()?.len() > 0,
        "main.wasm is required and must not be empty"
    );
    std::fs::create_dir_all(root.path().join("data"))?;
    std::fs::remove_file(root.path().join("archive.gz"))?;
    Ok(Bundle { root })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gzip.write_all(bytes).unwrap();
        gzip.finish().unwrap()
    }
    fn archive(entries: &[(&str, tar::EntryType, &[u8])]) -> Vec<u8> {
        let mut archive = tar::Builder::new(Vec::new());
        for (path, kind, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            // Deliberately bypass set_path's checks to exercise our own reader.
            header.as_mut_bytes()[..path.len()].copy_from_slice(path.as_bytes());
            header.set_entry_type(*kind);
            header.set_mode(0o644);
            header.set_size(bytes.len() as u64);
            header.set_cksum();
            archive.append(&header, *bytes).unwrap();
        }
        archive.into_inner().unwrap()
    }
    fn stage(bytes: &[u8]) -> TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("archive.gz"), bytes).unwrap();
        root
    }
    fn reject(bytes: &[u8], limit: u64, entries: usize) {
        let root = stage(bytes);
        let path = root.path().to_owned();
        assert!(unpack(root, limit, entries).is_err());
        assert!(!path.exists(), "failed extraction must remove staged files");
    }

    #[test]
    fn bundle_layout_and_lifetime() {
        let bytes = gzip(&archive(&[
            ("./", tar::EntryType::Directory, b""),
            ("./main.wasm", tar::EntryType::Regular, b"wasm"),
            ("./data/index.html", tar::EntryType::Regular, b"hello"),
        ]));
        let bundle = unpack(stage(&bytes), 16_384, 10).unwrap();
        assert_eq!(
            std::fs::read(bundle.data().join("index.html")).unwrap(),
            b"hello"
        );
        assert_eq!(std::fs::read(bundle.main()).unwrap(), b"wasm");
        let root = bundle.root.path().to_owned();
        assert!(!root.join("archive.gz").exists());
        drop(bundle);
        assert!(!root.exists());
        let bytes = gzip(&archive(&[("main.wasm", tar::EntryType::Regular, b"wasm")]));
        assert!(unpack(stage(&bytes), 16_384, 10).unwrap().data().is_dir());
    }

    #[test]
    fn rejects_paths_links_duplicates_and_nonfiles() {
        use tar::EntryType as T;
        for bad in [
            ("../escape", T::Regular, &b"bad"[..]),
            ("/main.wasm", T::Regular, b"bad"),
            ("data/../../escape", T::Regular, b"bad"),
            ("data\\escape", T::Regular, b"bad"),
            ("unexpected", T::Regular, b"bad"),
            ("data", T::Regular, b"bad"),
            ("main.wasm", T::Regular, b"duplicate"),
            ("data/link", T::Symlink, b""),
            ("data/link", T::Link, b""),
            ("data/device", T::Char, b""),
            ("data/pipe", T::Fifo, b""),
            ("data/dir", T::Directory, b"nonempty"),
        ] {
            reject(
                &gzip(&archive(&[("main.wasm", T::Regular, b"wasm"), bad])),
                16_384,
                10,
            );
        }
        reject(
            &gzip(&archive(&[("data/file", T::Regular, b"no main")])),
            16_384,
            10,
        );
        reject(
            &gzip(&archive(&[("main.wasm", T::Regular, b"")])),
            16_384,
            10,
        );
        reject(
            &gzip(&archive(&[("main.wasm", T::Directory, b"")])),
            16_384,
            10,
        );
    }

    #[test]
    fn validates_limits_and_gzip_footer() {
        use tar::EntryType as T;
        let tar = archive(&[
            ("main.wasm", T::Regular, b"wasm"),
            ("data/index", T::Regular, b"hello"),
        ]);
        let bytes = gzip(&tar);
        reject(&bytes, 1000, 10);
        reject(&bytes, 16_384, 1);
        reject(&bytes[..bytes.len() - 4], 16_384, 10);
        let mut corrupt = bytes.clone();
        let crc = corrupt.len() - 8;
        corrupt[crc] ^= 1;
        reject(&corrupt, 16_384, 10);
        let mut trailing = tar;
        trailing.extend_from_slice(b"hidden payload");
        reject(&gzip(&trailing), 16_384, 10);
    }

    #[test]
    fn supports_standard_long_path_metadata() {
        let mut archive = tar::Builder::new(Vec::new());
        let name = format!("data/{}", "long".repeat(40));
        for path in ["main.wasm", &name] {
            let mut header = tar::Header::new_gnu();
            header.set_size(4);
            header.set_mode(0o644);
            header.set_cksum();
            archive
                .append_data(&mut header, path, &b"test"[..])
                .unwrap();
        }
        let bytes = gzip(&archive.into_inner().unwrap());
        let bundle = unpack(stage(&bytes), 16_384, 10).unwrap();
        assert_eq!(
            std::fs::read(bundle.root.path().join(name)).unwrap(),
            b"test"
        );
    }
}
