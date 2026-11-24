use crate::{
    config::{Config, Spec},
    logs::{LogStream, Logs},
    proto::wafl::{Instance, InstanceAddRequest, InstanceRef, Phase, Tenant, instance_ref},
    runtime::Runtime,
    store::{Bundle, Store},
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use connectrpc::ConnectError;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Notify, Semaphore, oneshot, watch},
    task::JoinHandle,
};
use wasmtime::component::Component;

#[derive(Clone)]
pub struct Runner(Arc<Registry>);
struct Registry {
    entries: Mutex<Entries>,
    runtime: Arc<Runtime>,
    client: reqwest::Client,
    store: Store,
    slots: Arc<Semaphore>,
    closed: Arc<AtomicBool>,
    stopped: Notify,
}
#[derive(Default)]
struct Entries {
    instances: BTreeMap<[u8; 16], Arc<Execution>>,
    pending: BTreeSet<[u8; 16]>,
    aliases: BTreeMap<String, [u8; 16]>,
}
pub struct Execution {
    pub spec: Spec,
    bundle: Bundle,
    logs: Arc<Logs>,
    runtime: Arc<Runtime>,
    component: Component,
    state: Mutex<State>,
    changed: watch::Sender<()>,
    closed: Arc<AtomicBool>,
}
struct State {
    retired: bool,
    phase: Phase,
    generation: u64,
    active: u32,
    error: String,
    address: Option<SocketAddr>,
    activity: Instant,
    task: Option<JoinHandle<()>>,
}

/// A request owns this lease through the final response frame or cancellation.
pub struct Lease {
    app: Arc<Execution>,
    generation: u64,
    pub address: SocketAddr,
}

// Releases an unpublished instance and its ID on error or RPC cancellation.
struct Addition {
    runner: Runner,
    id: [u8; 16],
    alias: String,
    execution: Option<Arc<Execution>>,
    committed: bool,
}
impl Drop for Addition {
    fn drop(&mut self) {
        if !self.committed {
            let mut entries = self.runner.0.entries.lock().unwrap();
            entries.pending.remove(&self.id);
            entries.aliases.remove(&self.alias);
            drop(entries);
            if let Some(execution) = &self.execution {
                let _ = execution.retire(true);
            }
        }
    }
}

impl Runner {
    pub async fn new(config: &Config) -> Result<Self> {
        config.validate()?;
        Ok(Self(Arc::new(Registry {
            entries: Mutex::new(Entries::default()),
            runtime: Arc::new(Runtime::new()?),
            store: Store::new(&config.directory()?)?,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::limited(5))
                .referer(false)
                .build()?,
            slots: Arc::new(Semaphore::new(4)),
            closed: Arc::new(AtomicBool::new(false)),
            stopped: Notify::new(),
        })))
    }

    pub async fn add(&self, req: InstanceAddRequest) -> Result<Instance, ConnectError> {
        let mut spec = Spec::parse(req)?;
        let slot = Arc::new(self.0.slots.clone().try_acquire_owned().map_err(|_| {
            ConnectError::resource_exhausted("at most 4 concurrent Add operations")
        })?);
        let mut addition = {
            let mut entries = self.0.entries.lock().unwrap();
            if self.0.closed.load(Ordering::Acquire) {
                return Err(ConnectError::unavailable("runner is shutting down"));
            }
            if entries.instances.contains_key(&spec.id) || entries.pending.contains(&spec.id) {
                return Err(ConnectError::already_exists(
                    "instance ID already exists or is being added",
                ));
            }
            if entries.instances.len() + entries.pending.len() >= 128 {
                return Err(ConnectError::resource_exhausted(
                    "runner instance limit is 128",
                ));
            }
            if entries.aliases.contains_key(&spec.alias) {
                return Err(ConnectError::already_exists(
                    "instance alias already exists or is being added",
                ));
            }
            entries.pending.insert(spec.id);
            entries.aliases.insert(spec.alias.clone(), spec.id);
            Addition {
                runner: self.clone(),
                id: spec.id,
                alias: spec.alias.clone(),
                execution: None,
                committed: false,
            }
        };
        let create = async {
            let bundle = self
                .0
                .store
                .load(
                    &self.0.client,
                    spec.url.clone(),
                    &spec.manifest.bundle.digest,
                    slot.clone(),
                )
                .await?;
            spec.manifest.bundle.get_or_insert_default().digest = bundle.digest.clone();
            let engine = self.0.runtime.engine.clone();
            let compilation_slot = slot.clone();
            let (bundle, component) = tokio::task::spawn_blocking(move || {
                let _slot = compilation_slot;
                Component::from_file(&engine, bundle.main()).map(|component| (bundle, component))
            })
            .await
            .map_err(|_| ConnectError::internal("WASM compilation task failed"))?
            .map_err(|e| {
                ConnectError::invalid_argument(format!("invalid main.wasm component: {e}"))
            })?;
            let (changed, _) = watch::channel(());
            let execution = Arc::new(Execution {
                spec,
                bundle,
                logs: Logs::new(),
                component,
                runtime: self.0.runtime.clone(),
                changed,
                closed: self.0.closed.clone(),
                state: Mutex::new(State {
                    retired: false,
                    phase: Phase::Stopped,
                    generation: 0,
                    active: 0,
                    error: String::new(),
                    address: None,
                    activity: Instant::now(),
                    task: None,
                }),
            });
            addition.execution = Some(execution.clone());
            let lease = self
                .acquire(&execution)
                .await
                .map_err(|e| ConnectError::unavailable(format!("instance startup: {e:#}")))?;
            drop(lease);
            {
                let mut entries = self.0.entries.lock().unwrap();
                if self.0.closed.load(Ordering::Acquire) {
                    return Err(ConnectError::unavailable("runner is shutting down"));
                }
                entries.pending.remove(&addition.id);
                entries.instances.insert(addition.id, execution.clone());
                addition.committed = true;
            }
            Ok(execution.snapshot())
        };
        tokio::select! {
            result = create => result,
            _ = self.closing() => Err(ConnectError::unavailable("runner is shutting down")),
        }
    }

    async fn closing(&self) {
        let notified = self.0.stopped.notified();
        if self.0.closed.load(Ordering::Acquire) {
            return;
        }
        notified.await;
    }
    pub fn instance(&self, id: &[u8]) -> Option<Arc<Execution>> {
        let id: [u8; 16] = id.try_into().ok()?;
        self.0.entries.lock().unwrap().instances.get(&id).cloned()
    }
    pub fn resolve(&self, reference: &InstanceRef) -> Result<Option<[u8; 16]>, ConnectError> {
        match &reference.key {
            Some(instance_ref::Key::Id(id))
                if paydirt::has_domain(id, crate::config::INSTANCE_DOMAIN) =>
            {
                Ok(Some(id.as_slice().try_into().unwrap()))
            }
            Some(instance_ref::Key::Alias(alias)) if crate::config::valid_alias(alias) => {
                Ok(self.0.entries.lock().unwrap().aliases.get(alias).copied())
            }
            _ => Err(ConnectError::invalid_argument(
                "ref must contain a domain-7 ID or an alias without @",
            )),
        }
    }
    pub fn list(&self) -> Vec<Instance> {
        self.executions()
            .iter()
            .map(|instance| instance.snapshot())
            .collect()
    }
    fn executions(&self) -> Vec<Arc<Execution>> {
        self.0
            .entries
            .lock()
            .unwrap()
            .instances
            .values()
            .cloned()
            .collect()
    }
    pub async fn erase(&self, id: &[u8]) -> Result<bool, ConnectError> {
        let id: [u8; 16] = id
            .try_into()
            .map_err(|_| ConnectError::invalid_argument("invalid instance ID"))?;
        let (execution, task) = {
            let mut entries = self.0.entries.lock().unwrap();
            let Some(execution) = entries.instances.get(&id).cloned() else {
                if entries.pending.contains(&id) {
                    return Err(ConnectError::failed_precondition(
                        "instance is still being added",
                    ));
                }
                return Ok(false);
            };
            // Retire before removing the route: a previously obtained reference
            // cannot start another guest between Stop and removal.
            let task = execution
                .retire(false)
                .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
            entries.instances.remove(&id);
            entries.aliases.remove(&execution.spec.alias);
            (execution, task)
        };
        if let Some(task) = task {
            let _ = task.await;
        }
        drop(execution);
        Ok(true)
    }
    pub async fn acquire(&self, execution: &Arc<Execution>) -> Result<Lease> {
        ensure!(
            !self.0.closed.load(Ordering::Acquire),
            "runner is shutting down"
        );
        execution.acquire().await
    }
    pub async fn reap(&self) {
        for execution in self.executions() {
            let _ = execution.stop(true, false).await;
        }
    }
    pub(crate) fn drain(&self) {
        self.0.closed.store(true, Ordering::Release);
        self.0.stopped.notify_waiters();
        for execution in self.executions() {
            execution.logs.close();
        }
    }
    pub async fn shutdown(&self) {
        self.drain();
        let instances = std::mem::take(&mut self.0.entries.lock().unwrap().instances);
        for execution in instances.values() {
            if let Ok(Some(task)) = execution.retire(true) {
                let _ = task.await;
            }
        }
    }
}
impl Drop for Registry {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Release);
        for execution in self.entries.get_mut().unwrap().instances.values() {
            let _ = execution.retire(true);
        }
    }
}

impl Execution {
    pub fn snapshot(&self) -> Instance {
        let state = self.state.lock().unwrap();
        Instance {
            id: self.spec.id.to_vec(),
            tenant: Some(Tenant {
                id: self.spec.tenant.to_vec(),
                ..Default::default()
            })
            .into(),
            name: self.spec.name.clone(),
            alias: self.spec.alias.clone(),
            phase: state.phase.into(),
            generation: state.generation,
            active: state.active,
            error: state.error.clone(),
            manifest: Some(self.spec.manifest.clone()).into(),
            ..Default::default()
        }
    }

    async fn acquire(self: &Arc<Self>) -> Result<Lease> {
        // Subscribe before reading the state so readiness cannot be missed.
        let mut changed = self.changed.subscribe();
        let generation;
        {
            let mut state = self.state.lock().unwrap();
            ensure!(
                !self.closed.load(Ordering::Acquire),
                "runner is shutting down"
            );
            ensure!(!state.retired, "instance was erased");
            ensure!(state.phase != Phase::Stopping, "instance is stopping");
            ensure!(
                state.active < self.spec.max_active,
                "instance request limit reached"
            );
            if state.phase == Phase::Failed && state.activity.elapsed() < Duration::from_secs(1) {
                bail!("{}", state.error);
            }
            ensure!(
                state.phase != Phase::Failed || state.active == 0,
                "failed instance still has requests finishing"
            );
            if matches!(state.phase, Phase::Stopped | Phase::Failed) {
                state.generation += 1;
                state.phase = Phase::Starting;
                state.error.clear();
                state.address = None;
                let app = self.clone();
                let generation = state.generation;
                // The startup task owns its timeout; cancelling the first caller
                // neither cancels other callers nor leaves a stuck Starting state.
                state.task = Some(tokio::spawn(async move {
                    app.run(generation).await;
                }));
            }
            generation = state.generation;
            state.active += 1;
        }
        let mut lease = Lease {
            app: self.clone(),
            generation,
            address: SocketAddr::from(([127, 0, 0, 1], 0)),
        };
        loop {
            {
                let state = self.state.lock().unwrap();
                ensure!(
                    state.generation == generation,
                    "instance changed during startup"
                );
                match state.phase {
                    Phase::Running => {
                        lease.address = state.address.expect("running endpoint");
                        return Ok(lease);
                    }
                    Phase::Starting => {}
                    _ => bail!(
                        "{}",
                        if state.error.is_empty() {
                            "instance stopped"
                        } else {
                            &state.error
                        }
                    ),
                }
            }
            changed
                .changed()
                .await
                .context("instance supervisor ended")?;
        }
    }

    async fn run(self: Arc<Self>, generation: u64) {
        let (ready, receiver) = oneshot::channel();
        let data = self.bundle.data();
        let guest = self.runtime.execute(
            &self.component,
            &self.spec,
            &data,
            self.logs.clone(),
            generation,
            ready,
        );
        tokio::pin!(guest);
        let started = tokio::select! {
            result = &mut guest => Err(result.err().unwrap_or_else(|| anyhow!("guest exited before listening"))),
            endpoint = receiver => endpoint.context("guest did not open its declared port"),
            _ = tokio::time::sleep(Duration::from_millis(self.spec.startup_ms)) => Err(anyhow!("guest startup timed out")),
        };
        let result = match started {
            Ok(address) => {
                {
                    let mut state = self.state.lock().unwrap();
                    if state.generation != generation || state.phase != Phase::Starting {
                        return;
                    }
                    state.address = Some(address);
                    state.phase = Phase::Running;
                    state.activity = Instant::now();
                }
                self.changed.send_replace(());
                guest.await.err().unwrap_or_else(|| anyhow!("guest exited"))
            }
            Err(e) => e,
        };
        let mut state = self.state.lock().unwrap();
        if state.generation == generation && state.phase != Phase::Stopping && !state.retired {
            state.phase = Phase::Failed;
            state.error = format!("{result:#}");
            state.address = None;
            state.activity = Instant::now();
            state.task = None;
        }
        self.changed.send_replace(());
    }

    fn retire(&self, force: bool) -> Result<Option<JoinHandle<()>>> {
        let mut state = self.state.lock().unwrap();
        ensure!(force || state.active == 0, "instance has active requests");
        state.retired = true;
        state.phase = Phase::Stopped;
        state.address = None;
        self.logs.close();
        self.changed.send_replace(());
        let task = state.task.take();
        if let Some(task) = &task {
            task.abort();
        }
        Ok(task)
    }

    pub(crate) fn tail(
        &self,
        cursor: u64,
        limit: u32,
        follow: bool,
    ) -> Result<LogStream, ConnectError> {
        self.logs.tail(cursor, limit, follow)
    }

    pub async fn stop(self: &Arc<Self>, idle: bool, force: bool) -> Result<()> {
        let (task, generation) = {
            let mut state = self.state.lock().unwrap();
            if state.phase == Phase::Stopped {
                return Ok(());
            }
            if idle
                && (state.phase != Phase::Running
                    || state.active != 0
                    || state.activity.elapsed() < Duration::from_millis(self.spec.idle_ms))
            {
                return Ok(());
            }
            ensure!(force || state.active == 0, "instance has active requests");
            ensure!(
                state.phase != Phase::Stopping,
                "instance is already stopping"
            );
            state.phase = Phase::Stopping;
            state.address = None;
            (state.task.take(), state.generation)
        };
        self.changed.send_replace(());
        // Keep cleanup independent of the RPC caller: a cancelled Stop must not
        // leave a live guest or permanently prevent the next request from starting.
        let app = self.clone();
        tokio::spawn(async move {
            if let Some(task) = task {
                task.abort();
                let _ = task.await;
            }
            let mut state = app.state.lock().unwrap();
            if state.generation == generation {
                state.phase = Phase::Stopped;
                state.error.clear();
                state.activity = Instant::now();
            }
            app.changed.send_replace(());
        })
        .await?;
        Ok(())
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut state = self.app.state.lock().unwrap();
        if state.generation == self.generation {
            state.active -= 1;
            if state.active == 0 {
                state.activity = Instant::now();
            }
        }
    }
}
