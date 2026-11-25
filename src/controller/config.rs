use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, net::SocketAddr, path::Path};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub database: String,
    pub runners: Vec<String>,
    #[serde(default = "listen")]
    pub listen: SocketAddr,
    #[serde(default = "interval")]
    pub interval_ms: u64,
    #[serde(default = "fresh")]
    pub fresh_ms: u64,
    #[serde(default = "probe")]
    pub probe_ms: u64,
    #[serde(default = "operation")]
    pub operation_ms: u64,
    #[serde(default = "pending")]
    pub max_pending: usize,
    #[serde(default = "retiring")]
    pub max_retiring: usize,
    #[serde(default = "replicas")]
    pub max_replicas: u32,
    #[serde(default = "entrypoint")]
    pub entrypoint: String,
}
fn listen() -> SocketAddr {
    "127.0.0.1:8082".parse().unwrap()
}
fn interval() -> u64 {
    1000
}
fn fresh() -> u64 {
    10000
}
fn probe() -> u64 {
    2000
}
fn operation() -> u64 {
    120000
}
fn pending() -> usize {
    4
}
fn retiring() -> usize {
    32
}
fn replicas() -> u32 {
    32
}
fn entrypoint() -> String {
    "web".into()
}
impl Config {
    pub fn read(path: &Path) -> Result<Self> {
        let value: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.database.starts_with("postgres://") || self.database.starts_with("postgresql://"),
            "controller requires a PostgreSQL database URL"
        );
        ensure!(
            !self.runners.is_empty() && self.runners.len() <= 64,
            "configure 1..64 individual runner control endpoints"
        );
        let mut unique = BTreeSet::new();
        for endpoint in &self.runners {
            let url = reqwest::Url::parse(endpoint)?;
            ensure!(
                url.scheme() == "http"
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.path() == "/"
                    && url.query().is_none()
                    && url.fragment().is_none(),
                "runner endpoint must be an HTTP origin without credentials"
            );
            ensure!(unique.insert(url.to_string()), "duplicate runner endpoint");
        }
        ensure!(
            (100..=60000).contains(&self.interval_ms) && (100..=10000).contains(&self.probe_ms),
            "invalid controller interval/probe timeout"
        );
        ensure!(
            (1000..=600000).contains(&self.operation_ms)
                && self.fresh_ms
                    >= 3 * (self.interval_ms
                        + self.probe_ms * self.runners.len().div_ceil(8) as u64)
                && self.fresh_ms <= 600000,
            "fresh_ms must cover three observation cycles and be at most 600000"
        );
        ensure!(
            (1..=64).contains(&self.max_pending)
                && (1..=1000).contains(&self.max_replicas)
                && (1..=1000).contains(&self.max_retiring),
            "invalid controller scheduling limits"
        );
        ensure!(
            !self.entrypoint.is_empty()
                && self.entrypoint.len() <= 64
                && self
                    .entrypoint
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
            "invalid Traefik entrypoint"
        );
        Ok(())
    }
    pub fn signature(&self) -> String {
        let mut runners: Vec<_> = self
            .runners
            .iter()
            .map(|s| reqwest::Url::parse(s).unwrap().to_string())
            .collect();
        runners.sort();
        format!("{:x}", Sha256::digest(serde_json::to_vec(&serde_json::json!({"format":1,"runners":runners,"pending":self.max_pending,"retiring":self.max_retiring,"replicas":self.max_replicas,"fresh":self.fresh_ms,"entrypoint":self.entrypoint})).unwrap()))
    }
}
