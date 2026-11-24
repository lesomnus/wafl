use crate::{config::Capacity, proto::wafl::Usage};
use connectrpc::ConnectError;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub(crate) struct Admission {
    pub limits: Capacity,
    startups: Arc<Semaphore>,
    requests: Arc<Semaphore>,
    waiting: Arc<Semaphore>,
    executions: Mutex<(usize, u64)>,
}
pub(crate) struct Reservation {
    owner: Arc<Admission>,
    bytes: u64,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let mut current = self.owner.executions.lock().unwrap();
        current.0 -= 1;
        current.1 -= self.bytes;
    }
}
impl Admission {
    pub fn new(limits: Capacity) -> Arc<Self> {
        Arc::new(Self {
            startups: Arc::new(Semaphore::new(limits.startups)),
            requests: Arc::new(Semaphore::new(limits.requests)),
            waiting: Arc::new(Semaphore::new(limits.waiting)),
            executions: Mutex::new((0, 0)),
            limits,
        })
    }
    pub fn request(&self) -> Result<OwnedSemaphorePermit, ConnectError> {
        self.requests
            .clone()
            .try_acquire_owned()
            .map_err(|_| ConnectError::resource_exhausted("runner request capacity reached"))
    }
    pub fn reserve(self: &Arc<Self>, bytes: u64) -> Result<Reservation, ConnectError> {
        let mut current = self.executions.lock().unwrap();
        if current.0 >= self.limits.running
            || bytes > self.limits.memory_bytes.saturating_sub(current.1)
        {
            return Err(ConnectError::resource_exhausted(
                "runner execution or reserved memory capacity reached",
            ));
        }
        current.0 += 1;
        current.1 += bytes;
        Ok(Reservation {
            owner: self.clone(),
            bytes,
        })
    }
    pub async fn startup(&self) -> Result<OwnedSemaphorePermit, ConnectError> {
        if let Ok(slot) = self.startups.clone().try_acquire_owned() {
            return Ok(slot);
        }
        let _waiting = self
            .waiting
            .clone()
            .try_acquire_owned()
            .map_err(|_| ConnectError::resource_exhausted("runner startup queue is full"))?;
        tokio::time::timeout(
            Duration::from_millis(self.limits.queue_ms),
            self.startups.clone().acquire_owned(),
        )
        .await
        .map_err(|_| ConnectError::deadline_exceeded("runner startup queue timed out"))?
        .map_err(|_| ConnectError::unavailable("runner startup queue closed"))
    }
    pub fn usage(&self) -> Usage {
        let current = self.executions.lock().unwrap();
        Usage {
            running: current.0 as u64,
            memory_bytes: current.1,
            startups: (self.limits.startups - self.startups.available_permits()) as u64,
            requests: (self.limits.requests - self.requests.available_permits()) as u64,
            waiting: (self.limits.waiting - self.waiting.available_permits()) as u64,
            ..Default::default()
        }
    }
}
