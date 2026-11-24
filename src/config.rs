use crate::proto::wafl::{InstanceAddRequest, Manifest, Phase, asset, tenant_ref};
use anyhow::{Context, Result, ensure};
use connectrpc::ConnectError;
use serde::Deserialize;
use std::{collections::BTreeSet, net::SocketAddr, path::Path};

pub const INSTANCE_DOMAIN: u8 = 7;
pub const TENANT_DOMAIN: u8 = 1;

/// Process listeners only. Instances arrive through InstanceService.Add.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub listen: SocketAddr,
    pub control: SocketAddr,
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
            self.listen != self.control || self.listen.port() == 0,
            "ingress and control require separate listeners"
        );
        Ok(())
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
    pub name: String,
    pub manifest: Manifest,
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
        {
            return Err(invalid(
                "phase, generation, active and error are server-set",
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
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid(
                "bundle URL must be HTTP(S), without userinfo or fragment",
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
        Ok(Self {
            id,
            tenant,
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
        })
    }
}
