use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Deserializer};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub control: SocketAddr,
    pub apps: Vec<App>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub id: Id,
    pub tenant: Id,
    pub name: String,
    pub artifact: PathBuf,
    pub runtime: Runtime,
    pub abi: Abi,
    pub domain: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default = "idle_ms")]
    pub idle_ms: u64,
    #[serde(default = "startup_ms")]
    pub startup_ms: u64,
    #[serde(default = "memory_bytes")]
    pub memory_bytes: usize,
    #[serde(default = "max_active")]
    pub max_active: u32,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

pub const INSTANCE_DOMAIN: u8 = 7;
pub const TENANT_DOMAIN: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub [u8; 16]);

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        paydirt::parse_id(&value)
            .map(Self)
            .ok_or_else(|| serde::de::Error::custom("expected a Payday UUID"))
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    Wasmtime,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Abi {
    Wasip2,
}
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[default]
    Http,
    H2c,
}

/// An operator-provided TCP service, addressed by the guest through loopback.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub port: u16,
    pub target: SocketAddr,
}

fn idle_ms() -> u64 {
    60_000
}
fn startup_ms() -> u64 {
    10_000
}
fn memory_bytes() -> usize {
    64 * 1024 * 1024
}
fn max_active() -> u32 {
    128
}

impl Config {
    pub fn read(path: &Path) -> Result<Self> {
        let mut config: Self =
            serde_json::from_slice(&std::fs::read(path)?).context("invalid runner manifest")?;
        let base = path.parent().unwrap_or(Path::new("."));
        for app in &mut config.apps {
            app.artifact = base
                .join(&app.artifact)
                .canonicalize()
                .with_context(|| format!("artifact for {}", app.name))?;
        }
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.listen != self.control || self.listen.port() == 0,
            "ingress and control require separate listeners"
        );
        ensure!(!self.apps.is_empty(), "at least one app is required");
        let mut names = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut domains = BTreeSet::new();
        for app in &self.apps {
            ensure!(
                paydirt::has_domain(&app.id.0, INSTANCE_DOMAIN),
                "invalid Instance ID: expected domain {INSTANCE_DOMAIN}"
            );
            ensure!(
                paydirt::has_domain(&app.tenant.0, TENANT_DOMAIN),
                "invalid Tenant ID: expected domain {TENANT_DOMAIN}"
            );
            ensure!(ids.insert(app.id), "duplicate Instance ID");
            ensure!(
                !app.name.is_empty()
                    && app.name.len() <= 63
                    && app
                        .name
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
                "invalid app name"
            );
            ensure!(names.insert(&app.name), "duplicate app name {}", app.name);
            ensure!(
                valid_domain(&app.domain),
                "invalid domain {}: use a lowercase hostname without a port",
                app.domain
            );
            ensure!(
                domains.insert(&app.domain),
                "duplicate domain {}",
                app.domain
            );
            ensure!(app.port != 0, "guest port must be nonzero");
            ensure!(
                app.idle_ms > 0 && app.startup_ms > 0 && app.max_active > 0,
                "timeouts and max_active must be positive"
            );
            ensure!(
                app.memory_bytes >= 1024 * 1024,
                "memory_bytes must be at least 1 MiB"
            );
            ensure!(
                !app.env.contains_key("PORT"),
                "PORT is supplied from the app port"
            );
            for (key, value) in &app.env {
                ensure!(
                    !key.is_empty() && !key.contains(['=', '\0']) && !value.contains('\0'),
                    "invalid environment entry"
                );
            }
            let mut ports = BTreeSet::new();
            for binding in &app.bindings {
                ensure!(
                    binding.port != 0 && binding.port != app.port && ports.insert(binding.port),
                    "invalid or duplicate binding port"
                );
                ensure!(
                    binding.target.is_ipv4()
                        && binding.target.port() != 0
                        && !binding.target.ip().is_unspecified(),
                    "binding target requires a concrete IPv4 address and port"
                );
            }
        }
        Ok(())
    }
}

fn valid_domain(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        })
}
