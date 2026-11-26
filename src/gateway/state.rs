use super::{
    control::Control,
    traffic::{Admission, Gate},
};
use crate::proto::wafl::{App, Route, RouteTarget, Routing};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone, Debug)]
pub(super) enum Key {
    Host(String),
    Port(u32),
}
#[derive(Debug)]
pub(super) struct Failure(pub u16, pub &'static str);
#[derive(Default)]
struct Cache {
    routing: Routing,
    date: Option<Instant>,
}
#[derive(Default)]
struct Gates {
    all: HashMap<String, Arc<Gate>>,
    placements: HashMap<Vec<u8>, Arc<Gate>>,
}
pub(super) struct State {
    pub control: Control,
    cache: RwLock<Cache>,
    gates: Mutex<Gates>,
    pub stopping: AtomicBool,
    pub budget: Arc<Semaphore>,
    sequence: AtomicUsize,
}
pub(super) struct Target {
    pub group_key: u64,
    pub endpoint: RouteTarget,
    pub protocol: String,
    pub _admission: Admission,
    pub _permit: OwnedSemaphorePermit,
}
impl State {
    pub fn new(control: Control, limit: usize) -> Arc<Self> {
        Arc::new(Self {
            control,
            cache: RwLock::new(Cache::default()),
            gates: Mutex::new(Gates::default()),
            stopping: AtomicBool::new(false),
            budget: Arc::new(Semaphore::new(limit)),
            sequence: AtomicUsize::new(0),
        })
    }
    pub fn healthy(&self) -> bool {
        !self.stopping.load(Ordering::Acquire)
            && self
                .cache
                .read()
                .unwrap()
                .date
                .is_some_and(|d| d.elapsed() < Duration::from_secs(5))
    }
    pub async fn refresh(&self) -> anyhow::Result<()> {
        let routing = self.control.routes().await?;
        for route in &routing.routes {
            anyhow::ensure!(paydirt::has_domain(&route.app, 10), "invalid routed App ID");
            for target in &route.targets {
                anyhow::ensure!(
                    paydirt::has_domain(&target.placement, 11)
                        && target.address.parse::<std::net::SocketAddr>().is_ok(),
                    "invalid routing target"
                );
            }
        }
        let wanted: HashSet<_> = routing
            .routes
            .iter()
            .flat_map(|r| r.targets.iter().map(|t| t.placement.clone()))
            .collect();
        let mut gates = self.gates.lock().unwrap();
        for gate in gates.all.values() {
            if gate.placement.as_ref().is_some_and(|p| !wanted.contains(p)) {
                gate.close();
            }
        }
        gates.all.retain(|_, g| !g.finished());
        gates.placements.retain(|_, g| !g.closed());
        *self.cache.write().unwrap() = Cache {
            routing,
            date: Some(Instant::now()),
        };
        Ok(())
    }
    fn route(&self, key: &Key) -> Result<Route, Failure> {
        if !self.healthy() {
            return Err(Failure(503, "proxy routing unavailable"));
        }
        let cache = self.cache.read().unwrap();
        let route = cache
            .routing
            .routes
            .iter()
            .find(|r| match key {
                Key::Host(h) => r.port == 0 && r.hosts.contains(h),
                Key::Port(p) => r.port == *p,
            })
            .cloned()
            .ok_or(Failure(404, "App not found"))?;
        if route.suspended {
            return Err(Failure(503, "App is suspended"));
        }
        Ok(route)
    }
    fn gate(&self, route: &Route, placement: Option<Vec<u8>>) -> Result<Arc<Gate>, Failure> {
        let mut gates = self.gates.lock().unwrap();
        if let Some(ref p) = placement
            && let Some(g) = gates.placements.get(p)
            && !g.closed()
        {
            return Ok(g.clone());
        }
        if gates.all.len() >= 4096 {
            return Err(Failure(503, "observer capacity exhausted"));
        }
        let gate = Gate::new(&route.tenant, placement.clone(), &route.app);
        gates.all.insert(gate.id.clone(), gate.clone());
        if let Some(p) = placement {
            gates.placements.insert(p, gate.clone());
        }
        tokio::spawn(gate.clone().run(self.control.clone()));
        Ok(gate)
    }
    pub async fn target(&self, key: Key, deadline: Option<Instant>) -> Result<Target, Failure> {
        let permit = self
            .budget
            .clone()
            .try_acquire_owned()
            .map_err(|_| Failure(429, "proxy capacity exhausted"))?;
        let first = self.route(&key)?;
        let limit = deadline
            .unwrap_or_else(|| Instant::now() + Duration::from_millis(first.wait_ms.max(1)))
            .min(Instant::now() + Duration::from_millis(first.wait_ms.max(1)));
        let result =
            tokio::time::timeout_at(tokio::time::Instant::from_std(limit), async {
                let mut waiting = None;
                let mut wake = Instant::now() - Duration::from_secs(10);
                loop {
                    let route = self.route(&key)?;
                    if route.app != first.app {
                        return Err(Failure(503, "App was replaced"));
                    }
                    let offset = self.sequence.fetch_add(1, Ordering::Relaxed);
                    for n in 0..route.targets.len() {
                        let endpoint = &route.targets[(n + offset) % route.targets.len()];
                        let gate = self.gate(&route, Some(endpoint.placement.clone()))?;
                        if !gate.ready().await {
                            continue;
                        }
                        let current = self.route(&key)?;
                        if current.app != first.app {
                            return Err(Failure(503, "App was replaced"));
                        }
                        if !current.targets.iter().any(|t| {
                            t.placement == endpoint.placement && t.address == endpoint.address
                        }) {
                            continue;
                        }
                        if let Some(admission) = gate.enter() {
                            drop(waiting.take());
                            return Ok(Target {
                                group_key: gate.group_key,
                                endpoint: endpoint.clone(),
                                protocol: route.protocol,
                                _admission: admission,
                                _permit: permit,
                            });
                        }
                    }
                    if waiting.is_none() {
                        let gate = self.gate(&route, None)?;
                        waiting = Some(gate.waiting());
                        if !gate.ready().await {
                            return Err(Failure(
                                gate.failure(),
                                "activation registration unavailable",
                            ));
                        }
                    }
                    if wake.elapsed() >= Duration::from_secs(1) {
                        self.control
                            .rpc::<App>(
                                "AppService/Wake",
                                &route.tenant,
                                &json!({"ref":{"id":STANDARD.encode(&route.app)}}),
                            )
                            .await
                            .map_err(|_| Failure(503, "App activation unavailable"))?;
                        wake = Instant::now();
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await;
        result.unwrap_or(Err(Failure(504, "App startup deadline exceeded")))
    }
    pub fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
        for gate in self.gates.lock().unwrap().all.values() {
            gate.close();
        }
    }
    pub fn drained(&self) -> bool {
        self.gates
            .lock()
            .unwrap()
            .all
            .values()
            .all(|g| g.finished())
    }
}
