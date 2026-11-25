use crate::proto::wafl::{InstanceAddRequest, Manifest, Phase, asset, tenant_ref};
use anyhow::{Context, Result, ensure};
use connectrpc::ConnectError;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

pub const INSTANCE_DOMAIN: u8 = 7;
pub const TENANT_DOMAIN: u8 = 1;
pub const RESERVATION_DOMAIN: u8 = 9;

/// Process configuration. Instances arrive through InstanceService.Add.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub control: SocketAddr,
    #[serde(default)]
    pub directory: Option<PathBuf>,
    #[serde(default)]
    pub metrics: Option<Metrics>,
    #[serde(default)]
    pub bundle: BundleLimits,
    #[serde(default)]
    pub cache: Cache,
    #[serde(default)]
    pub capacity: Capacity,
    /// Direct guest sockets; disables the runner HTTP ingress and idle reaper.
    #[serde(default)]
    pub direct: Option<Direct>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Direct {
    pub listen: Ipv4Addr,
    pub advertise: Ipv4Addr,
    /// Durable reservations, separate from the disposable bundle cache.
    /// All runners advertising this IP must share this directory.
    pub directory: PathBuf,
    pub first: u16,
    pub last: u16,
}
impl Direct {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.directory.is_absolute(),
            "direct.directory must be an absolute durable state directory"
        );
        ensure!(
            self.first > 0 && self.first <= self.last,
            "direct.first..last must be a nonempty port range"
        );
        ensure!(
            !self.advertise.is_unspecified()
                && !self.advertise.is_multicast()
                && !self.advertise.is_broadcast(),
            "direct.advertise must be a concrete unicast IPv4 address"
        );
        ensure!(
            self.listen.is_unspecified() || self.listen == self.advertise,
            "direct.listen must be 0.0.0.0 or direct.advertise"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BundleLimits {
    pub download_bytes: u64,
    pub expanded_bytes: u64,
    pub entries: usize,
}
impl Default for BundleLimits {
    fn default() -> Self {
        Self {
            download_bytes: 64 * 1024 * 1024,
            expanded_bytes: 256 * 1024 * 1024,
            entries: 4096,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Cache {
    pub capacity_bytes: u64,
}
impl Default for Cache {
    fn default() -> Self {
        Self {
            capacity_bytes: 10 * 1024 * 1024 * 1024,
        }
    }
}

/// Admission budgets per process, independent of disk cache capacity.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Capacity {
    pub instances: usize,
    pub tombstones: usize,
    pub preparations: usize,
    pub startups: usize,
    pub running: usize,
    pub memory_bytes: u64,
    pub requests: usize,
    pub waiting: usize,
    pub queue_ms: u64,
    pub drain_ms: u64,
    pub compilations: usize,
}
impl Default for Capacity {
    fn default() -> Self {
        Self {
            instances: 128,
            tombstones: 65536,
            preparations: 4,
            startups: 4,
            running: 128,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            requests: 1024,
            waiting: 128,
            queue_ms: 10_000,
            drain_ms: 30_000,
            compilations: 32,
        }
    }
}
impl Capacity {
    fn validate(&self) -> Result<()> {
        for value in [
            self.instances,
            self.tombstones,
            self.preparations,
            self.startups,
            self.running,
            self.requests,
            self.waiting,
            self.compilations,
        ] {
            ensure!(
                value > 0 && value <= tokio::sync::Semaphore::MAX_PERMITS,
                "capacity counts must be positive and fit a semaphore"
            );
        }
        ensure!(
            self.memory_bytes >= 1024 * 1024,
            "capacity.memory_bytes must be at least 1 MiB"
        );
        ensure!(
            (1..=600_000).contains(&self.queue_ms) && (1..=600_000).contains(&self.drain_ms),
            "queue_ms and drain_ms must be 1..600000"
        );
        Ok(())
    }
}

/// Runner-owned OTLP/HTTP protobuf export; never passed to a guest manifest.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    /// Complete metrics URL, including /v1/metrics when using a Collector.
    pub endpoint: String,
    #[serde(default = "metrics_interval")]
    pub interval_ms: u64,
    #[serde(default = "metrics_timeout")]
    pub timeout_ms: u64,
}
fn metrics_interval() -> u64 {
    10_000
}
fn metrics_timeout() -> u64 {
    3_000
}

impl Metrics {
    fn validate(&self) -> Result<()> {
        let endpoint = reqwest::Url::parse(&self.endpoint).context("invalid metrics endpoint")?;
        ensure!(
            matches!(endpoint.scheme(), "http" | "https")
                && endpoint.host_str().is_some()
                && endpoint.username().is_empty()
                && endpoint.password().is_none()
                && endpoint.fragment().is_none(),
            "metrics.endpoint must be an HTTP(S) URL without userinfo or fragment"
        );
        ensure!(
            (100..=86_400_000).contains(&self.interval_ms),
            "metrics.interval_ms must be 100..86400000"
        );
        ensure!(
            (1..=5_000).contains(&self.timeout_ms),
            "metrics.timeout_ms must be 1..5000"
        );
        Ok(())
    }
}
impl Config {
    pub fn read(path: &Path) -> Result<Self> {
        let value: Self = serde_json::from_slice(&std::fs::read(path)?)
            .context("invalid runner configuration")?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.bundle.download_bytes > 0
                && self.bundle.expanded_bytes > 0
                && self.bundle.expanded_bytes < u64::MAX
                && self.bundle.entries > 0,
            "bundle limits must be positive and expanded_bytes less than u64::MAX"
        );
        ensure!(
            self.cache.capacity_bytes > 0,
            "cache.capacity_bytes must be positive"
        );
        ensure!(
            self.direct.is_some() || self.listen != self.control || self.listen.port() == 0,
            "ingress and control require separate listeners"
        );
        self.capacity.validate()?;
        if let Some(direct) = &self.direct {
            direct.validate()?;
            ensure!(
                !(direct.first..=direct.last).contains(&self.control.port()),
                "control port must be outside the direct endpoint range"
            );
            if let Some(directory) = &self.directory {
                ensure!(
                    !direct.directory.starts_with(directory)
                        && !directory.starts_with(&direct.directory),
                    "direct state and bundle cache directories must be separate"
                );
            }
        }
        if let Some(metrics) = &self.metrics {
            metrics.validate()?;
        }
        Ok(())
    }

    pub fn directory(&self) -> Result<PathBuf> {
        if let Some(path) = &self.directory {
            return Ok(path.clone());
        }
        if let Some(path) = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
        {
            return Ok(path.join("wafl"));
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or_else(|| anyhow::anyhow!("set a cache directory, XDG_CACHE_HOME, or HOME"))?;
        Ok(home.join(".cache/wafl"))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    Http,
    H2c,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub port: u16,
    pub target: SocketAddr,
}

/// Validated runtime settings derived from the generated Add request.
#[derive(Clone, Debug)]
pub struct Spec {
    pub id: [u8; 16],
    pub tenant: [u8; 16],
    pub alias: String,
    pub name: String,
    pub manifest: Manifest,
    pub(crate) secrets: crate::secrets::Values,
    pub url: reqwest::Url,
    pub port: u16,
    pub protocol: Protocol,
    pub idle_ms: u64,
    pub startup_ms: u64,
    pub memory_bytes: usize,
    pub max_active: u32,
    pub bindings: Vec<Binding>,
}
impl Spec {
    pub fn parse(req: InstanceAddRequest) -> Result<Self, ConnectError> {
        let invalid = ConnectError::invalid_argument;
        if req.phase != Phase::Unspecified
            || req.generation != 0
            || req.active != 0
            || !req.error.is_empty()
            || !req.session.is_empty()
            || req.revision != 0
            || req.draining
            || req.endpoint.as_option().is_some()
            || req.direct
        {
            return Err(invalid(
                "phase, generation, active, error, session, revision, draining, endpoint and direct are server-set",
            ));
        }
        let id = match req.id {
            None => paydirt::mint(INSTANCE_DOMAIN),
            Some(id) if paydirt::has_domain(&id, INSTANCE_DOMAIN) => id.try_into().unwrap(),
            _ => return Err(invalid("id must be a domain-7 Payday ID, or omitted")),
        };
        let tenant = match &req.tenant.key {
            Some(tenant_ref::Key::Id(id)) if paydirt::has_domain(id, TENANT_DOMAIN) => {
                id.clone().try_into().unwrap()
            }
            _ => return Err(invalid("tenant.id must be a domain-1 Payday ID")),
        };
        let alias = req.alias.unwrap_or_default();
        let alias = if alias.is_empty() {
            format!("i-{}", paydirt::format_id(&id).unwrap())
        } else {
            alias
        };
        if !valid_alias(&alias) {
            return Err(invalid(
                "alias must be 1..63 lowercase letters, digits or hyphens, starting with a letter",
            ));
        }
        if req.name.len() > 256 || req.name.contains('\0') {
            return Err(invalid("name must be at most 256 bytes with no NUL"));
        }
        let mut manifest = req
            .manifest
            .as_option()
            .cloned()
            .ok_or_else(|| invalid("manifest is required"))?;
        if manifest.runtime.is_empty() {
            manifest.runtime = "wasmtime".into();
        }
        if manifest.abi.is_empty() {
            manifest.abi = "wasip2".into();
        }
        if manifest.protocol.is_empty() {
            manifest.protocol = "http".into();
        }
        if manifest.runtime != "wasmtime" || manifest.abi != "wasip2" {
            return Err(invalid("runtime/abi must be wasmtime/wasip2"));
        }
        let protocol = match manifest.protocol.as_str() {
            "http" => Protocol::Http,
            "h2c" => Protocol::H2c,
            _ => return Err(invalid("protocol must be http or h2c")),
        };
        let port = u16::try_from(manifest.port)
            .ok()
            .filter(|p| *p != 0)
            .ok_or_else(|| invalid("manifest.port must be 1..65535"))?;
        let url = match &manifest.bundle.kind {
            Some(asset::Kind::Url(url)) => {
                reqwest::Url::parse(url).map_err(|_| invalid("invalid bundle URL"))?
            }
            _ => return Err(invalid("manifest.bundle.url is required")),
        };
        let source_valid = match url.scheme() {
            "http" | "https" => url.host_str().is_some(),
            "file" => url.to_file_path().is_ok_and(|p| p.is_absolute()) && url.query().is_none(),
            _ => false,
        };
        if !source_valid
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid(
                "bundle URL must be HTTP(S) or a local absolute file URL, without userinfo or fragment",
            ));
        }
        if !manifest.bundle.digest.is_empty()
            && !crate::store::valid_digest(&manifest.bundle.digest)
        {
            return Err(invalid(
                "bundle.digest must be sha256:<64 lowercase hexadecimal digits>",
            ));
        }
        if manifest.idle_ms == 0 {
            manifest.idle_ms = 60_000;
        }
        if manifest.startup_ms == 0 {
            manifest.startup_ms = 10_000;
        }
        if manifest.memory_bytes == 0 {
            manifest.memory_bytes = 64 * 1024 * 1024;
        }
        if manifest.max_active == 0 {
            manifest.max_active = 128;
        }
        if manifest.idle_ms > 7 * 24 * 60 * 60 * 1000 || manifest.startup_ms > 600_000 {
            return Err(invalid(
                "idle_ms must be at most 7 days and startup_ms at most 10 minutes",
            ));
        }
        let memory_bytes = usize::try_from(manifest.memory_bytes)
            .ok()
            .filter(|n| *n >= 1024 * 1024)
            .ok_or_else(|| invalid("memory_bytes must fit the host and be at least 1 MiB"))?;
        for (key, value) in &manifest.env {
            if key == "PORT" || key.is_empty() || key.contains(['=', '\0']) || value.contains('\0')
            {
                return Err(invalid(
                    "invalid environment entry; PORT is supplied by the runner",
                ));
            }
        }
        let mut ports = BTreeSet::new();
        let mut bindings = Vec::new();
        for value in &manifest.bindings {
            let binding_port = u16::try_from(value.port)
                .ok()
                .filter(|p| *p != 0 && *p != port)
                .ok_or_else(|| invalid("invalid binding port"))?;
            let target: SocketAddr = value
                .target
                .parse()
                .map_err(|_| invalid("invalid binding target"))?;
            if !ports.insert(binding_port)
                || !target.is_ipv4()
                || target.port() == 0
                || target.ip().is_unspecified()
            {
                return Err(invalid(
                    "binding requires a unique port and concrete IPv4 target",
                ));
            }
            bindings.push(Binding {
                port: binding_port,
                target,
            });
        }
        let secrets = crate::secrets::Values::parse(req.secrets, &manifest)?;
        Ok(Self {
            id,
            tenant,
            alias,
            name: req.name,
            port,
            protocol,
            url,
            bindings,
            idle_ms: manifest.idle_ms,
            startup_ms: manifest.startup_ms,
            memory_bytes,
            max_active: manifest.max_active,
            manifest,
            secrets,
        })
    }
}

pub fn valid_alias(value: &str) -> bool {
    (1..=63).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
