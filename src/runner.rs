use crate::{
    admission::{Admission, Reservation},
    compiler::{Compiled, Compiler},
    config::{Config, Spec},
    endpoint::{Claim, Ports},
    logs::{LogStream, Logs},
    proto::wafl::{
        Endpoint, Instance, InstanceAddRequest, InstanceRef, Phase, Tenant, instance_ref,
    },
    runtime::Runtime,
    store::{Bundle, Store},
    telemetry::{InstanceSample, Observe, Snapshot},
};
use anyhow::{Result, anyhow};
use connectrpc::ConnectError;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Notify, OwnedSemaphorePermit, Semaphore, oneshot, watch},
    task::JoinHandle,
};

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
    session: [u8; 16],
    revisions: Arc<AtomicU64>,
    admission: Arc<Admission>,
    compiler: Arc<Compiler>,
    addresses: Mutex<(SocketAddr, SocketAddr)>,
}
#[derive(Default)]
struct Entries {
    sealed: BTreeSet<[u8; 16]>,
    instances: BTreeMap<[u8; 16], Arc<Execution>>,
    pending: BTreeMap<[u8; 16], Pending>,
    aliases: BTreeMap<String, [u8; 16]>,
}
struct Pending {
    value: Instance,
    execution: Option<Arc<Execution>>,
}
impl Pending {
    fn snapshot(&self) -> Instance {
        if let Some(execution) = &self.execution {
            let mut value = execution.snapshot();
            // Running becomes observable only when Add commits the route.
            value.phase = Phase::Starting.into();
            value.endpoint = Default::default();
            value
        } else {
            self.value.clone()
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Condition {
    pub session: Option<[u8; 16]>,
    pub revision: Option<u64>,
}

pub struct Execution {
    pub spec: Spec,
    bundle: Bundle,
    logs: Arc<Logs>,
    runtime: Arc<Runtime>,
    component: Compiled,
    admission: Arc<Admission>,
    session: [u8; 16],
    revisions: Arc<AtomicU64>,
    state: Mutex<State>,
    changed: watch::Sender<()>,
    closed: Arc<AtomicBool>,
}
type DrainResult = watch::Receiver<Option<Result<(), ConnectError>>>;
struct State {
    revision: u64,
    draining: bool,
    drain: Option<DrainResult>,
    failure: Option<ConnectError>,
    retired: bool,
    phase: Phase,
    generation: u64,
    active: u32,
    error: String,
    address: Option<SocketAddr>,
    activity: Instant,
    task: Option<JoinHandle<()>>,
    claim: Option<Arc<Claim>>,
    releasing: bool,
}

/// A request owns this lease through the final response frame or cancellation.
pub struct Lease {
    _request: OwnedSemaphorePermit,
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
                let execution = execution.clone();
                if let Ok(task) = execution.retire(true) {
                    tokio::spawn(async move {
                        if let Some(task) = task {
                            let _ = task.await;
                        }
                        // Pending snapshots never publish endpoints. Once its
                        // guest is gone an uncommitted Add can reclaim the slot.
                        let claim = execution.state.lock().unwrap().claim.clone();
                        if let Some(claim) = claim {
                            let _ = claim.release().await;
                        }
                    });
                }
            }
        }
    }
}

impl Runner {
    pub async fn new(config: &Config) -> Result<Self> {
        config.validate()?;
        let directory = config.directory()?;
        let session = paydirt::mint(8);
        let direct = match config.direct.clone() {
            Some(direct) => {
                let cache = directory.clone();
                Some(
                    tokio::task::spawn_blocking(move || {
                        let ports = Ports::new(direct, session)?;
                        std::fs::create_dir_all(&cache)?;
                        let cache = std::fs::canonicalize(cache)?;
                        let state = std::fs::canonicalize(&ports.config.directory)?;
                        anyhow::ensure!(
                            !state.starts_with(&cache) && !cache.starts_with(&state),
                            "direct state and bundle cache directories must be separate"
                        );
                        Ok::<_, anyhow::Error>(ports)
                    })
                    .await??,
                )
            }
            None => None,
        };
        let limits = config.bundle.clone();
        let cache = config.cache.clone();
        let store =
            tokio::task::spawn_blocking(move || Store::new(&directory, limits, &cache)).await??;
        let runtime = Arc::new(Runtime::new(direct)?);
        let compiler = Compiler::new(runtime.engine.clone(), config.capacity.compilations);
        Ok(Self(Arc::new(Registry {
            entries: Mutex::new(Entries::default()),
            runtime,
            store,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::limited(5))
                .referer(false)
                .build()?,
            slots: Arc::new(Semaphore::new(config.capacity.preparations)),
            closed: Arc::new(AtomicBool::new(false)),
            stopped: Notify::new(),
            session,
            revisions: Arc::new(AtomicU64::new(1)),
            admission: Admission::new(config.capacity.clone()),
            compiler,
            addresses: Mutex::new((config.listen, config.control)),
        })))
    }

    pub async fn add(&self, req: InstanceAddRequest) -> Result<Instance, ConnectError> {
        if self.direct() && (req.manifest.idle_ms != 0 || req.manifest.max_active != 0) {
            return Err(ConnectError::invalid_argument(
                "direct mode requires idle_ms=max_active=0; gateways own request accounting and idle retirement",
            ));
        }
        let mut spec = Spec::parse(req)?;
        if self.direct() {
            spec.idle_ms = 0;
            spec.manifest.idle_ms = 0;
            spec.manifest.max_active = 0;
        }
        let slot = Arc::new(self.0.slots.clone().try_acquire_owned().map_err(|_| {
            ConnectError::resource_exhausted("runner preparation capacity reached")
        })?);
        let mut addition = {
            let mut entries = self.0.entries.lock().unwrap();
            if self.0.closed.load(Ordering::Acquire) {
                return Err(ConnectError::unavailable("runner is shutting down"));
            }
            if entries.sealed.contains(&spec.id) {
                return Err(ConnectError::failed_precondition(
                    "instance ID is sealed for this runner session",
                ));
            }
            if entries.instances.contains_key(&spec.id) || entries.pending.contains_key(&spec.id) {
                return Err(ConnectError::already_exists(
                    "instance ID already exists or is being added",
                ));
            }
            if entries.instances.len() + entries.pending.len() >= self.0.admission.limits.instances
            {
                return Err(ConnectError::resource_exhausted(
                    "runner instance capacity reached",
                ));
            }
            if entries.aliases.contains_key(&spec.alias) {
                return Err(ConnectError::already_exists(
                    "instance alias already exists or is being added",
                ));
            }
            let revision = self.0.revisions.fetch_add(1, Ordering::Relaxed);
            entries.pending.insert(
                spec.id,
                Pending {
                    value: Instance {
                        id: spec.id.to_vec(),
                        tenant: Some(Tenant {
                            id: spec.tenant.to_vec(),
                            ..Default::default()
                        })
                        .into(),
                        alias: spec.alias.clone(),
                        name: spec.name.clone(),
                        manifest: Some(spec.manifest.clone()).into(),
                        phase: Phase::Preparing.into(),
                        direct: self.direct(),
                        session: self.0.session.to_vec(),
                        revision,
                        ..Default::default()
                    },
                    execution: None,
                },
            );
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
            let compiler = self.0.compiler.clone();
            let compilation_slot = slot.clone();
            let (bundle, component) = tokio::task::spawn_blocking(move || {
                let _slot = compilation_slot;
                compiler
                    .compile(&bundle.main())
                    .map(|component| (bundle, component))
            })
            .await
            .map_err(|_| ConnectError::internal("WASM compilation task failed"))??;
            let (changed, _) = watch::channel(());
            let execution = Arc::new(Execution {
                spec,
                bundle,
                logs: Logs::new(),
                component,
                admission: self.0.admission.clone(),
                session: self.0.session,
                revisions: self.0.revisions.clone(),
                runtime: self.0.runtime.clone(),
                changed,
                closed: self.0.closed.clone(),
                state: Mutex::new(State {
                    revision: self.0.revisions.fetch_add(1, Ordering::Relaxed),
                    draining: false,
                    drain: None,
                    failure: None,
                    retired: false,
                    phase: Phase::Stopped,
                    generation: 0,
                    active: 0,
                    error: String::new(),
                    address: None,
                    activity: Instant::now(),
                    task: None,
                    claim: None,
                    releasing: false,
                }),
            });
            addition.execution = Some(execution.clone());
            self.0
                .entries
                .lock()
                .unwrap()
                .pending
                .get_mut(&addition.id)
                .unwrap()
                .execution = Some(execution.clone());
            if self.0.entries.lock().unwrap().sealed.contains(&addition.id) {
                return Err(ConnectError::failed_precondition(
                    "instance ID was sealed during preparation",
                ));
            }
            let lease = self.acquire(&execution).await?;
            drop(lease);
            {
                let mut entries = self.0.entries.lock().unwrap();
                if self.0.closed.load(Ordering::Acquire) {
                    return Err(ConnectError::unavailable("runner is shutting down"));
                }
                if entries.sealed.contains(&addition.id) {
                    return Err(ConnectError::failed_precondition(
                        "instance ID was sealed during startup",
                    ));
                }
                execution.revise(&mut execution.state.lock().unwrap());
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
    pub fn snapshot(&self, id: &[u8]) -> Option<Instance> {
        let id: [u8; 16] = id.try_into().ok()?;
        let entries = self.0.entries.lock().unwrap();
        entries
            .instances
            .get(&id)
            .map(|e| e.snapshot())
            .or_else(|| entries.pending.get(&id).map(Pending::snapshot))
    }
    pub fn list(&self) -> Vec<Instance> {
        let entries = self.0.entries.lock().unwrap();
        entries
            .instances
            .values()
            .map(|e| e.snapshot())
            .chain(entries.pending.values().map(Pending::snapshot))
            .collect()
    }
    pub(crate) fn observe(&self) -> Observe {
        let registry = Arc::downgrade(&self.0);
        Arc::new(move || {
            let registry = registry.upgrade()?;
            let (instances, pending) = {
                let entries = registry.entries.lock().unwrap();
                (
                    entries.instances.values().cloned().collect::<Vec<_>>(),
                    entries.pending.len(),
                )
            };
            let instances = instances
                .iter()
                .map(|instance| {
                    let state = instance.state.lock().unwrap();
                    InstanceSample {
                        id: paydirt::format_id(&instance.spec.id).unwrap(),
                        phase: match state.phase {
                            Phase::Stopped => "stopped",
                            Phase::Starting => "starting",
                            Phase::Running => "running",
                            Phase::Failed => "failed",
                            Phase::Stopping => "stopping",
                            _ => "unspecified",
                        },
                        active: if registry.runtime.direct.is_some() {
                            None
                        } else {
                            Some(state.active as u64)
                        },
                    }
                })
                .collect();
            Some(Snapshot {
                instances,
                pending: pending as u64,
                usage: registry.admission.usage(),
                compilations: registry.compiler.count(),
                compile_hits: registry.compiler.hits.load(Ordering::Relaxed),
                compile_misses: registry.compiler.misses.load(Ordering::Relaxed),
                accepting: !registry.closed.load(Ordering::Acquire),
                preparing: (registry.admission.limits.preparations
                    - registry.slots.available_permits()) as u64,
            })
        })
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
        self.erase_condition(id, Condition::default()).await
    }
    pub async fn erase_condition(
        &self,
        id: &[u8],
        condition: Condition,
    ) -> Result<bool, ConnectError> {
        self.check_session(condition.session)?;
        let id: [u8; 16] = id
            .try_into()
            .map_err(|_| ConnectError::invalid_argument("invalid instance ID"))?;
        let (execution, task) = {
            let mut entries = self.0.entries.lock().unwrap();
            let Some(execution) = entries.instances.get(&id).cloned() else {
                if entries.pending.contains_key(&id) {
                    return Err(ConnectError::failed_precondition(
                        "instance is still being added",
                    ));
                }
                return Ok(false);
            };
            // Retire before removing the route: a previously obtained reference
            // cannot start another guest between Stop and removal.
            let task = execution.retire_condition(false, condition)?;
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
    pub async fn acquire(&self, execution: &Arc<Execution>) -> Result<Lease, ConnectError> {
        execution.acquire(false, Condition::default()).await
    }
    pub fn direct(&self) -> bool {
        self.0.runtime.direct.is_some()
    }
    pub(crate) fn ports(&self) -> Result<Arc<Ports>, ConnectError> {
        self.0.runtime.direct.clone().ok_or_else(|| {
            ConnectError::failed_precondition("reservation management requires direct mode")
        })
    }
    pub async fn start(
        &self,
        execution: &Arc<Execution>,
        condition: Condition,
    ) -> Result<Lease, ConnectError> {
        execution.acquire(true, condition).await
    }
    pub fn check_session(&self, session: Option<[u8; 16]>) -> Result<(), ConnectError> {
        if session.is_some_and(|s| s != self.0.session) {
            return Err(ConnectError::failed_precondition("runner session changed"));
        }
        Ok(())
    }
    pub fn session(&self) -> [u8; 16] {
        self.0.session
    }
    pub fn seal(&self, id: &[u8]) -> Result<(), ConnectError> {
        let id: [u8; 16] = <[u8; 16]>::try_from(id)
            .ok()
            .filter(|id| paydirt::has_domain(id, 7))
            .ok_or_else(|| ConnectError::invalid_argument("expected a domain-7 Instance ID"))?;
        let mut entries = self.0.entries.lock().unwrap();
        if entries.sealed.contains(&id) {
            return Ok(());
        }
        if entries.sealed.len() >= self.0.admission.limits.tombstones {
            return Err(ConnectError::resource_exhausted(
                "runner tombstone capacity reached",
            ));
        }
        entries.sealed.insert(id);
        Ok(())
    }
    pub fn drain_timeout(&self) -> Duration {
        Duration::from_millis(self.0.admission.limits.drain_ms)
    }
    pub fn addresses(&self, ingress: SocketAddr, control: SocketAddr) {
        *self.0.addresses.lock().unwrap() = (ingress, control);
    }
    pub fn status(&self) -> crate::proto::wafl::Runner {
        let limits = &self.0.admission.limits;
        let mut usage = self.0.admission.usage();
        let entries = self.0.entries.lock().unwrap();
        usage.instances = (entries.instances.len() + entries.pending.len()) as u64;
        usage.tombstones = entries.sealed.len() as u64;
        usage.preparations = (limits.preparations - self.0.slots.available_permits()) as u64;
        usage.compilations = self.0.compiler.count();
        let addresses = self.0.addresses.lock().unwrap();
        crate::proto::wafl::Runner {
            id: self.0.session.to_vec(),
            alias: "self".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            runtime: "wasmtime".into(),
            abi: "wasip2".into(),
            ingress: if self.direct() {
                String::new()
            } else {
                addresses.0.to_string()
            },
            control: addresses.1.to_string(),
            accepting: !self.0.closed.load(Ordering::Acquire),
            capabilities: [
                "http",
                "h2c",
                "grpc",
                if self.direct() {
                    "direct-endpoint-v1"
                } else {
                    "proxy-ingress"
                },
                if self.direct() {
                    "external-drain-v1"
                } else {
                    "drain"
                },
                "command-conditions",
                "sealed-instance-v1",
                "cache-v2",
                "outbound-tcp",
            ]
            .map(String::from)
            .into_iter()
            .chain(self.direct().then(|| "reservation-recovery-v1".to_owned()))
            .collect(),
            capacity: Some(crate::proto::wafl::Capacity {
                instances: limits.instances as u64,
                tombstones: limits.tombstones as u64,
                preparations: limits.preparations as u64,
                startups: limits.startups as u64,
                running: limits.running as u64,
                memory_bytes: limits.memory_bytes,
                requests: limits.requests as u64,
                waiting: limits.waiting as u64,
                queue_ms: limits.queue_ms,
                drain_ms: limits.drain_ms,
                compilations: limits.compilations as u64,
                ..Default::default()
            })
            .into(),
            usage: Some(usage).into(),
            ..Default::default()
        }
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
            direct: self.runtime.direct.is_some(),
            endpoint: state
                .claim
                .as_ref()
                .map(|claim| Endpoint {
                    address: claim.address.to_string(),
                    protocol: self.spec.manifest.protocol.clone(),
                    ..Default::default()
                })
                .into(),
            error: state.error.clone(),
            session: self.session.to_vec(),
            revision: state.revision,
            draining: state.draining,
            manifest: Some(self.spec.manifest.clone()).into(),
            ..Default::default()
        }
    }

    fn check(&self, state: &State, condition: Condition) -> Result<(), ConnectError> {
        if condition.session.is_some_and(|s| s != self.session) {
            return Err(ConnectError::failed_precondition("runner session changed"));
        }
        if condition.revision.is_some_and(|r| r != state.revision) {
            return Err(ConnectError::aborted("instance revision changed"));
        }
        Ok(())
    }
    fn revise(&self, state: &mut State) {
        state.revision = self.revisions.fetch_add(1, Ordering::Relaxed);
    }
    async fn acquire(
        self: &Arc<Self>,
        explicit: bool,
        condition: Condition,
    ) -> Result<Lease, ConnectError> {
        let request = self.admission.request()?;
        let mut changed = self.changed.subscribe();
        let generation;
        {
            let mut state = self.state.lock().unwrap();
            self.check(&state, condition)?;
            if self.closed.load(Ordering::Acquire) {
                return Err(ConnectError::unavailable("runner is shutting down"));
            }
            if state.retired {
                return Err(ConnectError::not_found("instance was erased"));
            }
            if self.runtime.direct.is_some()
                && (state.releasing
                    || (state.claim.is_some() && (state.draining || state.phase != Phase::Running)))
            {
                return Err(ConnectError::failed_precondition(
                    "withdraw the previous direct endpoint and Release before restarting",
                ));
            }
            if state.phase == Phase::Stopping
                || state.drain.is_some()
                || (state.draining && !explicit)
            {
                return Err(ConnectError::failed_precondition(
                    "instance is draining or stopping",
                ));
            }
            if state.active >= self.spec.max_active {
                return Err(ConnectError::resource_exhausted(
                    "instance request limit reached",
                ));
            }
            if state.phase == Phase::Failed && state.activity.elapsed() < Duration::from_secs(1) {
                return Err(state
                    .failure
                    .clone()
                    .unwrap_or_else(|| ConnectError::unavailable(state.error.clone())));
            }
            if matches!(state.phase, Phase::Stopped | Phase::Failed) {
                if state.active != 0 {
                    return Err(ConnectError::failed_precondition(
                        "previous execution still has requests finishing",
                    ));
                }
                let reservation = self.admission.reserve(self.spec.manifest.memory_bytes)?;
                state.generation += 1;
                state.phase = Phase::Starting;
                state.error.clear();
                state.failure = None;
                state.address = None;
                self.revise(&mut state);
                let app = self.clone();
                let generation = state.generation;
                // The attempt belongs to the supervisor, not its first caller.
                state.task = Some(tokio::spawn(async move {
                    app.run(generation, reservation).await;
                }));
            }
            if explicit && state.draining {
                state.draining = false;
                self.revise(&mut state);
            }
            generation = state.generation;
            state.active += 1;
        }
        self.changed.send_replace(());
        let mut lease = Lease {
            _request: request,
            app: self.clone(),
            generation,
            address: SocketAddr::from(([127, 0, 0, 1], 0)),
        };
        loop {
            {
                let state = self.state.lock().unwrap();
                if state.generation != generation {
                    return Err(ConnectError::aborted("instance changed during startup"));
                }
                match state.phase {
                    Phase::Running => {
                        lease.address = state.address.expect("running endpoint");
                        return Ok(lease);
                    }
                    Phase::Starting => {}
                    _ => {
                        return Err(state
                            .failure
                            .clone()
                            .unwrap_or_else(|| ConnectError::unavailable("instance stopped")));
                    }
                }
            }
            changed
                .changed()
                .await
                .map_err(|_| ConnectError::unavailable("instance supervisor ended"))?;
        }
    }
    async fn execute(&self, generation: u64) -> Result<(), ConnectError> {
        let startup = self.admission.startup().await?;
        let (ready, receiver) = oneshot::channel();
        let data = self.bundle.data();
        let guest = self.runtime.execute(
            self.component.component(),
            &self.spec,
            &data,
            self.logs.clone(),
            generation,
            ready,
        );
        tokio::pin!(guest);
        let ready = tokio::select! {
            result = &mut guest => return Err(ConnectError::unavailable(format!("guest startup: {:#}", result.err().unwrap_or_else(|| anyhow!("guest exited before listening"))))),
            endpoint = receiver => endpoint.map_err(|_| ConnectError::unavailable("guest did not open its declared port"))?,
            _ = tokio::time::sleep(Duration::from_millis(self.spec.startup_ms)) => return Err(ConnectError::deadline_exceeded("guest startup timed out")),
        };
        {
            let mut state = self.state.lock().unwrap();
            if state.generation != generation || state.phase != Phase::Starting {
                return Ok(());
            }
            if let Some(claim) = &ready.claim {
                claim.publish();
            }
            state.claim = ready.claim;
            state.address = Some(ready.address);
            state.phase = Phase::Running;
            state.activity = Instant::now();
            self.revise(&mut state);
        }
        drop(startup);
        self.changed.send_replace(());
        let error = guest.await.err().unwrap_or_else(|| anyhow!("guest exited"));
        Err(ConnectError::unavailable(format!(
            "guest execution: {error:#}"
        )))
    }
    async fn run(self: Arc<Self>, generation: u64, reservation: Reservation) {
        let result = self.execute(generation).await;
        // Guest future (memory/sockets) is gone before capacity becomes reusable.
        drop(reservation);
        let mut state = self.state.lock().unwrap();
        if state.generation == generation && state.phase != Phase::Stopping && !state.retired {
            state.phase = Phase::Failed;
            let error = result
                .err()
                .unwrap_or_else(|| ConnectError::unavailable("guest stopped"));
            state.error = error.message.clone().unwrap_or_default();
            state.failure = Some(error);
            state.address = None;
            state.activity = Instant::now();
            state.task = None;
            self.revise(&mut state);
        }
        self.changed.send_replace(());
    }
    fn retire(&self, force: bool) -> Result<Option<JoinHandle<()>>, ConnectError> {
        self.retire_condition(force, Condition::default())
    }
    fn retire_condition(
        &self,
        force: bool,
        condition: Condition,
    ) -> Result<Option<JoinHandle<()>>, ConnectError> {
        let mut state = self.state.lock().unwrap();
        self.check(&state, condition)?;
        if !force && (state.claim.is_some() || state.releasing) {
            return Err(ConnectError::failed_precondition(
                "Drain and Release the direct endpoint before Erase",
            ));
        }
        if !force && (state.active != 0 || state.drain.is_some()) {
            return Err(ConnectError::failed_precondition(
                "instance has active requests or drain",
            ));
        }
        state.retired = true;
        state.draining = true;
        state.phase = Phase::Stopped;
        state.address = None;
        self.revise(&mut state);
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
    pub async fn stop(self: &Arc<Self>, idle: bool, force: bool) -> Result<(), ConnectError> {
        self.stop_condition(idle, force, Condition::default(), false)
            .await
    }
    pub async fn stop_condition(
        self: &Arc<Self>,
        idle: bool,
        force: bool,
        condition: Condition,
        drain: bool,
    ) -> Result<(), ConnectError> {
        let (task, generation) = {
            let mut state = self.state.lock().unwrap();
            self.check(&state, condition)?;
            if idle && self.runtime.direct.is_some() {
                return Ok(());
            }
            if !force && (state.claim.is_some() || state.releasing) {
                return Err(ConnectError::failed_precondition(
                    "direct endpoints require Drain then Release after gateway drain",
                ));
            }
            if state.retired {
                return Err(ConnectError::not_found("instance was erased"));
            }
            if state.drain.is_some() && !drain && !force {
                if idle {
                    return Ok(());
                }
                return Err(ConnectError::failed_precondition(
                    "instance drain is in progress",
                ));
            }
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
            if !force && state.active != 0 {
                return Err(ConnectError::failed_precondition(
                    "instance has active requests",
                ));
            }
            if state.phase == Phase::Stopping {
                return Err(ConnectError::failed_precondition(
                    "instance is already stopping",
                ));
            }
            state.phase = Phase::Stopping;
            state.address = None;
            self.revise(&mut state);
            (state.task.take(), state.generation)
        };
        self.changed.send_replace(());
        let app = self.clone();
        tokio::spawn(async move {
            if let Some(task) = task {
                task.abort();
                let _ = task.await;
            }
            let mut state = app.state.lock().unwrap();
            if state.generation == generation && !state.retired {
                state.phase = Phase::Stopped;
                state.error.clear();
                state.failure = None;
                state.activity = Instant::now();
                app.revise(&mut state);
            }
            app.changed.send_replace(());
        })
        .await
        .map_err(|_| ConnectError::internal("stop supervisor failed"))?;
        Ok(())
    }
    pub async fn drain(
        self: &Arc<Self>,
        timeout_ms: u64,
        force: bool,
        condition: Condition,
    ) -> Result<(), ConnectError> {
        if self.runtime.direct.is_some() {
            if timeout_ms != 0 || force {
                return Err(ConnectError::invalid_argument(
                    "direct Drain only withdraws publication; timeout_ms and force must be zero; acknowledge gateway drain with Release",
                ));
            }
            let mut state = self.state.lock().unwrap();
            self.check(&state, condition)?;
            if state.retired {
                return Err(ConnectError::not_found("instance was erased"));
            }
            if matches!(state.phase, Phase::Starting | Phase::Stopping) || state.releasing {
                return Err(ConnectError::failed_precondition(
                    "instance lifecycle operation is in progress",
                ));
            }
            if !state.draining {
                state.draining = true;
                self.revise(&mut state);
            }
            self.changed.send_replace(());
            return Ok(());
        }
        let timeout_ms = if timeout_ms == 0 {
            self.admission.limits.drain_ms
        } else {
            timeout_ms
        };
        if timeout_ms > 600_000 {
            return Err(ConnectError::invalid_argument(
                "drain timeout_ms must be at most 600000",
            ));
        }
        let mut result = {
            let mut state = self.state.lock().unwrap();
            self.check(&state, condition)?;
            if state.retired {
                return Err(ConnectError::not_found("instance was erased"));
            }
            if state.phase == Phase::Stopping {
                return Err(ConnectError::failed_precondition("instance is stopping"));
            }
            if let Some(result) = &state.drain {
                result.clone()
            } else {
                state.draining = true;
                self.revise(&mut state);
                let (sender, receiver) = watch::channel(None);
                state.drain = Some(receiver.clone());
                let app = self.clone();
                // Cancelling the RPC does not reopen admission or abandon cleanup.
                tokio::spawn(async move {
                    let mut changed = app.changed.subscribe();
                    let wait = async {
                        loop {
                            {
                                let state = app.state.lock().unwrap();
                                if state.retired {
                                    return Err(ConnectError::not_found("instance was erased"));
                                }
                                if state.active == 0 {
                                    return Ok(());
                                }
                            }
                            changed.changed().await.map_err(|_| {
                                ConnectError::unavailable("instance supervisor ended")
                            })?;
                        }
                    };
                    let outcome =
                        match tokio::time::timeout(Duration::from_millis(timeout_ms), wait).await {
                            Ok(Ok(())) => {
                                app.stop_condition(false, false, Condition::default(), true)
                                    .await
                            }
                            Ok(Err(e)) => Err(e),
                            Err(_) if force => {
                                app.stop_condition(false, true, Condition::default(), true)
                                    .await
                            }
                            Err(_) => Err(ConnectError::deadline_exceeded(
                                "drain timed out; admission remains closed",
                            )),
                        };
                    app.state.lock().unwrap().drain = None;
                    sender.send_replace(Some(outcome));
                    app.changed.send_replace(());
                });
                receiver
            }
        };
        self.changed.send_replace(());
        loop {
            if let Some(value) = result.borrow().clone() {
                return value;
            }
            result
                .changed()
                .await
                .map_err(|_| ConnectError::internal("drain supervisor ended"))?;
        }
    }

    pub async fn release(
        self: &Arc<Self>,
        generation: u64,
        condition: Condition,
    ) -> Result<(), ConnectError> {
        if self.runtime.direct.is_none()
            || condition.session.is_none()
            || condition.revision.is_none()
            || generation == 0
        {
            return Err(ConnectError::failed_precondition(
                "Release requires direct mode, generation, wafl-session and wafl-revision",
            ));
        }
        let (task, claim) = {
            let mut state = self.state.lock().unwrap();
            self.check(&state, condition)?;
            if state.retired {
                return Err(ConnectError::not_found("instance was erased"));
            }
            if state.generation != generation {
                return Err(ConnectError::aborted("instance generation changed"));
            }
            if !state.draining
                || state.releasing
                || matches!(state.phase, Phase::Starting | Phase::Stopping)
                || state.active != 0
            {
                return Err(ConnectError::failed_precondition(
                    "Drain and finish management waiters before Release",
                ));
            }
            let Some(claim) = state.claim.clone() else {
                return Ok(());
            };
            state.releasing = true;
            state.phase = Phase::Stopping;
            state.address = None;
            self.revise(&mut state);
            (state.task.take(), claim)
        };
        self.changed.send_replace(());
        let app = self.clone();
        // Complete independently of the RPC connection. The old socket must be
        // gone before another execution can acquire its durable reservation.
        tokio::spawn(async move {
            if let Some(task) = task {
                task.abort();
                let _ = task.await;
            }
            let result = claim
                .release()
                .await
                .map_err(|e| ConnectError::internal(format!("release endpoint: {e:#}")));
            let mut state = app.state.lock().unwrap();
            if state.generation == generation {
                if result.is_ok() {
                    state.claim = None;
                }
                state.releasing = false;
                state.phase = Phase::Stopped;
                app.revise(&mut state);
            }
            app.changed.send_replace(());
            result
        })
        .await
        .map_err(|_| ConnectError::internal("endpoint release supervisor failed"))?
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
        self.app.changed.send_replace(());
    }
}
