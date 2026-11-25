use super::*;
use crate::secrets::{COUNT, VALUE_BYTES, Values, env_name};
use cap_std::{ambient_authority, fs::Dir};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path},
};

pub(super) fn validate(
    bindings: &[Secret],
    manifest: &Manifest,
    tenant: &[u8],
    owned: bool,
    config: &Config,
) -> Result<(), ConnectError> {
    let invalid = ConnectError::invalid_argument;
    if bindings.len() > COUNT {
        return Err(invalid("at most 64 secret bindings are allowed"));
    }
    let mut names = BTreeSet::new();
    for binding in bindings {
        if !env_name(&binding.env)
            || manifest.env.contains_key(&binding.env)
            || !names.insert(&binding.env)
        {
            return Err(invalid(
                "invalid or duplicate secret env, reserved PORT, or collision with manifest.env",
            ));
        }
        match &binding.source {
            Some(secret::Source::File(file)) if !owned => {
                if file.is_empty()
                    || file.len() > 4096
                    || file.contains('\0')
                    || !Path::new(file)
                        .components()
                        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
                {
                    return Err(invalid(
                        "secret file must be a relative path without parent traversal",
                    ));
                }
                if !config
                    .secrets
                    .files
                    .contains_key(&paydirt::format_id(tenant).unwrap_or_default())
                {
                    return Err(invalid("no secret directory configured for this tenant"));
                }
            }
            Some(secret::Source::SecretKeyRef(key)) if owned => {
                if key.name.is_empty()
                    || key.name.len() > 253
                    || !key.name.split('.').all(|s| {
                        !s.is_empty()
                            && s.len() <= 63
                            && !s.starts_with('-')
                            && !s.ends_with('-')
                            && s.bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    })
                    || key.key.is_empty()
                    || key.key.len() > 253
                    || !key
                        .key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
                {
                    return Err(invalid("invalid secretKeyRef name or key"));
                }
            }
            _ => {
                return Err(invalid(
                    "use file sources for ordinary Apps and secretKeyRef for Kubernetes Apps",
                ));
            }
        }
    }
    Ok(())
}

pub(super) struct Resolver {
    files: BTreeMap<String, Arc<Dir>>,
    pub kubernetes: Option<kube::Client>,
}
impl Resolver {
    pub async fn open(config: &Config) -> anyhow::Result<Self> {
        let mut files = BTreeMap::new();
        for (tenant, path) in &config.secrets.files {
            files.insert(
                tenant.clone(),
                Arc::new(
                    Dir::open_ambient_dir(path, ambient_authority())
                        .map_err(|_| anyhow::anyhow!("cannot open configured secret directory"))?,
                ),
            );
        }
        let kubernetes = if let Some(config) = &config.providers.kubernetes {
            Some(super::kubernetes::client(config).await?)
        } else {
            None
        };
        Ok(Self { files, kubernetes })
    }

    async fn values(
        &self,
        p: &Placement,
        owner: Option<AppSource>,
    ) -> Result<Values, ConnectError> {
        let unavailable = || {
            ConnectError::failed_precondition(
                "secret source is unavailable, invalid, or not permitted",
            )
        };
        let mut values = BTreeMap::new();
        let mut objects: BTreeMap<String, Option<k8s_openapi::api::core::v1::Secret>> =
            BTreeMap::new();
        for binding in &p.secrets {
            let bytes = match &binding.source {
                Some(secret::Source::File(path)) if owner.is_none() => {
                    let dir = self
                        .files
                        .get(&paydirt::format_id(&p.tenant.id).unwrap_or_default())
                        .ok_or_else(unavailable)?
                        .clone();
                    let path = path.clone();
                    Some(
                        tokio::task::spawn_blocking(move || read_file(&dir, &path))
                            .await
                            .map_err(|_| unavailable())??,
                    )
                }
                Some(secret::Source::SecretKeyRef(key)) => {
                    let owner = owner.as_ref().ok_or_else(unavailable)?;
                    let client = self.kubernetes.clone().ok_or_else(unavailable)?;
                    if !objects.contains_key(&key.name) {
                        let api = kube::Api::<k8s_openapi::api::core::v1::Secret>::namespaced(
                            client,
                            &owner.namespace,
                        );
                        objects.insert(
                            key.name.clone(),
                            api.get_opt(&key.name).await.map_err(|_| unavailable())?,
                        );
                    }
                    let value = objects[&key.name]
                        .as_ref()
                        .and_then(|s| s.data.as_ref())
                        .and_then(|m| m.get(&key.key));
                    match value {
                        Some(value) => Some(value.0.clone()),
                        None if key.optional => None,
                        None => return Err(unavailable()),
                    }
                }
                _ => return Err(unavailable()),
            };
            if let Some(bytes) = bytes {
                if bytes.len() > VALUE_BYTES {
                    return Err(unavailable());
                }
                let value = String::from_utf8(bytes).map_err(|_| unavailable())?;
                values.insert(binding.env.clone(), value);
            }
        }
        Values::parse(values, &p.manifest)
    }
}

fn read_file(dir: &Dir, path: &str) -> Result<Vec<u8>, ConnectError> {
    let unavailable = || ConnectError::failed_precondition("secret file is unavailable or invalid");
    // Capability-relative opens prevent ../ and symlink escapes. Nonblocking
    // open lets us reject FIFOs/devices before attempting a bounded read.
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = dir.open_with(path, &options).map_err(|_| unavailable())?;
    let metadata = file.metadata().map_err(|_| unavailable())?;
    if !metadata.is_file() || metadata.len() > VALUE_BYTES as u64 {
        return Err(unavailable());
    }
    let mut bytes = Vec::new();
    file.take(VALUE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable())?;
    if bytes.len() > VALUE_BYTES {
        return Err(unavailable());
    }
    Ok(bytes)
}

impl Control {
    pub(super) async fn secret_values(&self, p: &Placement) -> Result<Values, ConnectError> {
        if p.secrets.is_empty() {
            return Ok(Values::default());
        }
        let id = p.app.id.clone();
        let owner: Option<AppSource> = self
            .db
            .transaction(move |tx| {
                let app: App = decode(call(tx, "App", "Get", json!({"ref":{"id":encode(&id)}}))?)?;
                app.source
                    .as_option()
                    .map(|s| {
                        decode(call(
                            tx,
                            "AppSource",
                            "Get",
                            json!({"ref":{"id":encode(&s.id)}}),
                        )?)
                    })
                    .transpose()
            })
            .await?;
        validate(
            &p.secrets,
            &p.manifest,
            &p.tenant.id,
            owner.is_some(),
            &self.config,
        )?;
        if let Some(owner) = &owner {
            let valid = self.config.providers.kubernetes.as_ref().is_some_and(|k| {
                owner.provider == "kubernetes"
                    && owner.cluster == k.cluster
                    && owner.tenant.id == p.tenant.id
                    && k.namespaces
                        .get(&owner.namespace)
                        .and_then(|v| paydirt::parse_id(v))
                        .is_some_and(|id| id.as_slice() == p.tenant.id)
            });
            if !valid {
                return Err(ConnectError::failed_precondition(
                    "secret namespace binding changed",
                ));
            }
        }
        tokio::time::timeout(Duration::from_secs(10), self.secrets.values(p, owner))
            .await
            .map_err(|_| ConnectError::deadline_exceeded("secret resolution timed out"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_reads_are_bounded_regular_and_confined() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("token"), b"bytes\n").unwrap();
        std::fs::write(root.path().join("empty"), b"").unwrap();
        std::fs::write(root.path().join("large"), vec![0; VALUE_BYTES + 1]).unwrap();
        std::fs::write(outside.path().join("secret"), b"outside").unwrap();
        let dir = Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap();
        assert_eq!(read_file(&dir, "token").unwrap(), b"bytes\n");
        assert_eq!(read_file(&dir, "empty").unwrap(), b"");
        assert!(read_file(&dir, "large").is_err());
        assert!(read_file(&dir, ".").is_err());
        assert!(read_file(&dir, "../token").is_err());
        assert!(read_file(&dir, outside.path().join("secret").to_str().unwrap()).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(outside.path().join("secret"), root.path().join("escape"))
                .unwrap();
            std::os::unix::fs::symlink("token", root.path().join("internal")).unwrap();
            assert!(read_file(&dir, "escape").is_err());
            assert_eq!(read_file(&dir, "internal").unwrap(), b"bytes\n");
        }
        #[cfg(target_os = "linux")]
        {
            let path = std::ffi::CString::new(root.path().join("pipe").to_str().unwrap()).unwrap();
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
            assert!(read_file(&dir, "pipe").is_err());
        }
    }
    #[tokio::test]
    async fn resolution_rejects_invalid_text_and_total_size() {
        let root = tempfile::tempdir().unwrap();
        let tenant = paydirt::parse_id("00000000-0000-8000-8001-000000000001").unwrap();
        let mut config: Config =
            serde_json::from_value(json!({"database":"unused", "runners":[]})).unwrap();
        config
            .secrets
            .files
            .insert(paydirt::format_id(&tenant).unwrap(), root.path().to_owned());
        let resolver = Resolver::open(&config).await.unwrap();
        let placement = |files: Vec<(&str, &str)>| Placement {
            tenant: Some(Tenant {
                id: tenant.to_vec(),
                ..Default::default()
            })
            .into(),
            secrets: files
                .into_iter()
                .map(|(env, file)| Secret {
                    env: env.into(),
                    source: Some(secret::Source::File(file.into())),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        std::fs::write(root.path().join("value"), [255]).unwrap();
        assert!(
            resolver
                .values(&placement(vec![("TOKEN", "value")]), None)
                .await
                .is_err()
        );
        std::fs::write(root.path().join("value"), "a\0b").unwrap();
        assert!(
            resolver
                .values(&placement(vec![("TOKEN", "value")]), None)
                .await
                .is_err()
        );
        std::fs::write(root.path().join("value"), "a".repeat(VALUE_BYTES)).unwrap();
        assert!(
            resolver
                .values(
                    &placement(vec![
                        ("A", "value"),
                        ("B", "value"),
                        ("C", "value"),
                        ("D", "value"),
                        ("E", "value")
                    ]),
                    None
                )
                .await
                .is_err()
        );
    }
}
