use crate::{
    config::{self, Config},
    proto::wafl::{Instance, Phase},
    runtime::Runtime,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{oneshot, watch},
    task::JoinHandle,
};
use wasmtime::component::Component;

#[derive(Clone)]
pub struct Runner(Arc<Registry>);
struct Registry {
    apps: BTreeMap<String, Arc<App>>,
    domains: BTreeMap<String, String>,
    closed: Arc<AtomicBool>,
}
pub struct App {
    pub spec: config::App,
    runtime: Arc<Runtime>,
    component: Component,
    state: Mutex<State>,
    changed: watch::Sender<()>,
    closed: Arc<AtomicBool>,
}
struct State {
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
    app: Arc<App>,
    generation: u64,
    pub address: SocketAddr,
}

impl Runner {
    pub async fn new(config: &Config) -> Result<Self> {
        config.validate()?;
        let runtime = Arc::new(Runtime::new()?);
        let mut components = BTreeMap::new();
        let mut apps = BTreeMap::new();
        let mut domains = BTreeMap::new();
        let closed = Arc::new(AtomicBool::new(false));
        for spec in &config.apps {
            let path = spec
                .artifact
                .canonicalize()
                .with_context(|| format!("artifact for {}", spec.name))?;
            let component = if let Some(component) = components.get(&path) {
                Component::clone(component)
            } else {
                let engine = runtime.engine.clone();
                let file = path.clone();
                let component =
                    tokio::task::spawn_blocking(move || Component::from_file(&engine, file))
                        .await??;
                components.insert(path, component.clone());
                component
            };
            let (changed, _) = watch::channel(());
            let app = Arc::new(App {
                spec: spec.clone(),
                runtime: runtime.clone(),
                closed: closed.clone(),
                component,
                changed,
                state: Mutex::new(State {
                    phase: Phase::Stopped,
                    generation: 0,
                    active: 0,
                    error: String::new(),
                    address: None,
                    activity: Instant::now(),
                    task: None,
                }),
            });
            domains.insert(spec.domain.clone(), spec.name.clone());
            apps.insert(spec.name.clone(), app);
        }
        Ok(Self(Arc::new(Registry {
            apps,
            domains,
            closed,
        })))
    }

    pub fn app(&self, name: &str) -> Option<Arc<App>> {
        self.0.apps.get(name).cloned()
    }
    pub fn domain(&self, domain: &str) -> Option<Arc<App>> {
        self.0.domains.get(domain).and_then(|name| self.app(name))
    }
    pub fn list(&self) -> Vec<Instance> {
        self.0.apps.values().map(|app| app.snapshot()).collect()
    }
    pub async fn acquire(&self, app: &Arc<App>) -> Result<Lease> {
        ensure!(
            !self.0.closed.load(Ordering::Acquire),
            "runner is shutting down"
        );
        app.acquire().await
    }
    pub async fn reap(&self) {
        for app in self.0.apps.values() {
            let _ = app.stop(true, false).await;
        }
    }
    pub async fn shutdown(&self) {
        self.0.closed.store(true, Ordering::Release);
        for app in self.0.apps.values() {
            let _ = app.stop(false, true).await;
        }
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        for app in self.apps.values() {
            if let Some(task) = app.state.lock().unwrap().task.take() {
                task.abort();
            }
        }
    }
}

impl App {
    pub fn snapshot(&self) -> Instance {
        let state = self.state.lock().unwrap();
        Instance {
            name: self.spec.name.clone(),
            phase: state.phase.into(),
            generation: state.generation,
            active: state.active,
            error: state.error.clone(),
            domain: self.spec.domain.clone(),
            port: u32::from(self.spec.port),
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
        let guest = self.runtime.execute(&self.component, &self.spec, ready);
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
        if state.generation == generation && state.phase != Phase::Stopping {
            state.phase = Phase::Failed;
            state.error = format!("{result:#}");
            state.address = None;
            state.activity = Instant::now();
            state.task = None;
        }
        self.changed.send_replace(());
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
