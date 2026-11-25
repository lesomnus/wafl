//! Share compiled code across bundles (data and manifests do not affect code).
use connectrpc::ConnectError;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
};
use wasmtime::{Engine, component::Component};

pub(crate) struct Compiler {
    engine: Engine,
    retained: usize,
    entries: Mutex<BTreeMap<[u8; 32], Arc<Entry>>>,
    clock: AtomicU64,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
}
struct Entry {
    value: OnceLock<Result<Component, String>>,
    used: AtomicU64,
}
pub(crate) struct Compiled {
    entry: Option<Arc<Entry>>,
    owner: Weak<Compiler>,
}
impl Compiled {
    pub fn component(&self) -> &Component {
        self.entry
            .as_ref()
            .unwrap()
            .value
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
    }
}
impl Drop for Compiled {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            self.entry.as_ref().unwrap().used.store(
                owner.clock.fetch_add(1, Ordering::Relaxed),
                Ordering::Relaxed,
            );
            self.entry.take();
            owner.prune();
        }
    }
}
impl Compiler {
    pub fn new(engine: Engine, retained: usize) -> Arc<Self> {
        Arc::new(Self {
            engine,
            retained,
            entries: Mutex::new(BTreeMap::new()),
            clock: AtomicU64::new(1),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        })
    }
    // Runs inside the Add blocking worker, retaining its source pin and slot.
    pub fn compile(self: &Arc<Self>, path: &Path) -> Result<Compiled, ConnectError> {
        let bytes = std::fs::read(path)
            .map_err(|e| ConnectError::internal(format!("read main.wasm: {e}")))?;
        let key: [u8; 32] = Sha256::digest(&bytes).into();
        let entry = {
            let mut entries = self.entries.lock().unwrap();
            let entry = entries
                .entry(key)
                .or_insert_with(|| {
                    Arc::new(Entry {
                        value: OnceLock::new(),
                        used: AtomicU64::new(0),
                    })
                })
                .clone();
            entry.used.store(
                self.clock.fetch_add(1, Ordering::Relaxed),
                Ordering::Relaxed,
            );
            entry
        };
        let compiled = Compiled {
            entry: Some(entry.clone()),
            owner: Arc::downgrade(self),
        };
        let mut initialized = false;
        let result = entry.value.get_or_init(|| {
            initialized = true;
            self.misses.fetch_add(1, Ordering::Relaxed);
            Component::new(&self.engine, &bytes)
                .map_err(|e| format!("invalid main.wasm component: {e:#}"))
        });
        if !initialized {
            self.hits.fetch_add(1, Ordering::Relaxed);
        }
        if let Err(error) = result {
            let error = ConnectError::invalid_argument(error.clone());
            drop(entry);
            drop(compiled);
            return Err(error);
        }
        self.prune();
        Ok(compiled)
    }
    fn prune(&self) {
        let mut entries = self.entries.lock().unwrap();
        let mut unused: Vec<_> = entries
            .iter()
            .filter(|(_, e)| Arc::strong_count(e) == 1)
            .map(|(k, e)| (*k, e.used.load(Ordering::Relaxed)))
            .collect();
        unused.sort_by_key(|(_, used)| *used);
        let excess = unused.len().saturating_sub(self.retained);
        for (key, _) in unused.into_iter().take(excess) {
            entries.remove(&key);
        }
    }
    pub fn count(&self) -> u64 {
        self.entries.lock().unwrap().len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_compilation_coalesces_and_unused_retention_is_bounded() {
        let root = std::path::PathBuf::from(
            std::env::var_os("WAFL_TEST_GUEST_DIR").expect("run scripts/test.sh"),
        );
        let engine = crate::runtime::Runtime::new(None).unwrap().engine.clone();
        let compiler = Compiler::new(engine, 1);
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|_| {
                let compiler = compiler.clone();
                let barrier = barrier.clone();
                let path = root.join("socket.wasm");
                std::thread::spawn(move || {
                    barrier.wait();
                    compiler.compile(&path).unwrap()
                })
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(compiler.misses.load(Ordering::Relaxed), 1);
        assert_eq!(compiler.hits.load(Ordering::Relaxed), 1);
        let grpc = compiler.compile(&root.join("grpc.wasm")).unwrap();
        assert_eq!(compiler.count(), 2);
        drop(grpc);
        assert_eq!(
            compiler.count(),
            2,
            "live references are retained beyond the unused cache budget"
        );
        drop(results);
        assert_eq!(compiler.count(), 1);
        drop(compiler.compile(&root.join("socket.wasm")).unwrap());
        assert_eq!(compiler.misses.load(Ordering::Relaxed), 2);
        drop(compiler.compile(&root.join("grpc.wasm")).unwrap());
        assert_eq!(compiler.misses.load(Ordering::Relaxed), 3);
        assert_eq!(compiler.count(), 1);
        let invalid = tempfile::tempdir().unwrap();
        for bytes in [b"invalid1", b"invalid2", b"invalid3"] {
            let path = invalid.path().join("main.wasm");
            std::fs::write(&path, bytes).unwrap();
            assert!(compiler.compile(&path).is_err());
            assert_eq!(compiler.count(), 1);
        }
    }
}
