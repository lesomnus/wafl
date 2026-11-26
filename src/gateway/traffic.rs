use super::control::{Control, Rejected};
use crate::proto::wafl::Traffic;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Notify;

#[derive(Default)]
struct Counts {
    active: u32,
    started: u64,
    nanos: u128,
    date: Option<Instant>,
    sequence: u64,
    registered: bool,
    closed: bool,
    finished: bool,
}
impl Counts {
    fn integrate(&mut self) {
        let now = Instant::now();
        if let Some(date) = self.date {
            self.nanos += date.elapsed().as_nanos() * u128::from(self.active);
        }
        self.date = Some(now);
    }
}

static GROUP: AtomicU64 = AtomicU64::new(1);
pub(super) struct Gate {
    pub group_key: u64,
    pub id: String,
    failure: AtomicU16,
    pub placement: Option<Vec<u8>>,
    tenant: String,
    request: Value,
    counts: Mutex<Counts>,
    changed: Notify,
}
impl Gate {
    pub fn new(tenant: &str, placement: Option<Vec<u8>>, app: &[u8]) -> Arc<Self> {
        let mut id = *uuid::Uuid::new_v4().as_bytes();
        id[6] = (id[6] & 15) | 0x80;
        id[8] = 0x80;
        id[9] = 13;
        let id = STANDARD.encode(id);
        let mut request = json!({"id":id,"session":uuid::Uuid::new_v4().to_string()});
        if let Some(ref p) = placement {
            request["placement"] = json!({"id":STANDARD.encode(p)});
        } else {
            request["app"] = json!({"id":STANDARD.encode(app)});
        }
        Arc::new(Self {
            failure: AtomicU16::new(503),
            group_key: GROUP.fetch_add(1, Ordering::Relaxed),
            id,
            placement,
            tenant: tenant.into(),
            request,
            counts: Mutex::new(Counts::default()),
            changed: Notify::new(),
        })
    }
    pub fn close(&self) {
        let mut c = self.counts.lock().unwrap();
        c.closed = true;
        self.changed.notify_waiters();
    }
    pub fn failure(&self) -> u16 {
        self.failure.load(Ordering::Relaxed)
    }
    pub fn closed(&self) -> bool {
        self.counts.lock().unwrap().closed
    }
    pub fn finished(&self) -> bool {
        self.counts.lock().unwrap().finished
    }
    pub fn enter(self: &Arc<Self>) -> Option<Admission> {
        let mut c = self.counts.lock().unwrap();
        if !c.registered || c.closed {
            return None;
        }
        c.integrate();
        c.active += 1;
        c.started += 1;
        Some(Admission(self.clone()))
    }
    pub async fn ready(&self) -> bool {
        loop {
            let notify = self.changed.notified();
            {
                let c = self.counts.lock().unwrap();
                if c.closed {
                    return false;
                }
                if c.registered {
                    return true;
                }
            }
            tokio::select! { _ = notify => {}, _ = tokio::time::sleep(Duration::from_millis(100)) => {} }
        }
    }
    pub async fn run(self: Arc<Self>, control: Control) {
        // This task owns registration through completion, even when the request
        // which created the gate is cancelled while Add is in flight.
        let mut uncertain = false;
        loop {
            match control
                .rpc::<Traffic>("TrafficService/Add", &self.tenant, &self.request)
                .await
            {
                Ok(value) => {
                    let mut c = self.counts.lock().unwrap();
                    c.registered = true;
                    c.closed |= value.draining || value.closed;
                    if self.placement.is_none() {
                        c.active = if c.closed { 0 } else { 1 };
                        c.started = 1;
                        c.date = Some(Instant::now());
                    }
                    self.changed.notify_waiters();
                    break;
                }
                Err(error) => {
                    log::warn!("observer {} registration: {error}", self.id);
                    let status = error.downcast_ref::<Rejected>().map(|e| e.0);
                    if status == Some(429) {
                        self.failure.store(429, Ordering::Relaxed);
                    }
                    let rejected = matches!(status, Some(400 | 404 | 409 | 412 | 429));
                    // A registration may have committed before a lost response.
                    // Get also recovers an observer whose Pod has since vanished.
                    if let Ok(value) = control
                        .rpc::<Traffic>(
                            "TrafficService/Get",
                            &self.tenant,
                            &json!({"ref":{"id":self.id}}),
                        )
                        .await
                    {
                        if value.session != self.request["session"].as_str().unwrap_or("") {
                            self.close();
                            return;
                        }
                        let mut c = self.counts.lock().unwrap();
                        c.registered = true;
                        c.closed |= value.draining || value.closed;
                        if self.placement.is_none() {
                            c.active = if c.closed { 0 } else { 1 };
                            c.started = 1;
                            c.date = Some(Instant::now());
                        }
                        self.changed.notify_waiters();
                        break;
                    }
                    // Never open a socket after uncertain registration. A failed
                    // attempt is closed locally; retry its receipt in background.
                    self.close();
                    if rejected && !uncertain {
                        self.counts.lock().unwrap().finished = true;
                        return;
                    }
                    uncertain = true;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        loop {
            let (sequence, active, started, busy, closed) = {
                let mut c = self.counts.lock().unwrap();
                c.integrate();
                c.sequence += 1;
                (
                    c.sequence,
                    c.active,
                    c.started,
                    (c.nanos / 1_000_000).min(u128::from(u64::MAX)) as u64,
                    c.closed,
                )
            };
            let request = json!({"ref":{"id":self.id},"sequence":sequence.to_string(),"active":active,"started":started.to_string(),"busyMs":busy.to_string(),"closed":closed});
            match control
                .rpc::<Traffic>("TrafficService/Report", &self.tenant, &request)
                .await
            {
                Ok(value) => {
                    if value.draining || value.closed {
                        self.close();
                    }
                    if closed && active == 0 && value.closed && value.active == 0 {
                        self.counts.lock().unwrap().finished = true;
                        self.changed.notify_waiters();
                        return;
                    }
                }
                Err(error) => log::warn!("observer {} report: {error}", self.id),
            }
            tokio::time::sleep(Duration::from_millis(if self.closed() { 100 } else { 500 })).await;
        }
    }
    pub fn waiting(self: &Arc<Self>) -> Waiting {
        Waiting(self.clone())
    }
}

pub(super) struct Admission(Arc<Gate>);
impl Drop for Admission {
    fn drop(&mut self) {
        let mut c = self.0.counts.lock().unwrap();
        c.integrate();
        c.active -= 1;
        self.0.changed.notify_waiters();
    }
}
pub(super) struct Waiting(Arc<Gate>);
impl Drop for Waiting {
    fn drop(&mut self) {
        let mut c = self.0.counts.lock().unwrap();
        c.integrate();
        c.active = 0;
        c.closed = true;
        self.0.changed.notify_waiters();
    }
}
