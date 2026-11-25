use super::*;
use wafl_runner::services::wafl::{AppServiceClient, PlacementServiceClient, TrafficServiceClient};

#[derive(Subcommand)]
pub(super) enum AppCommand {
    Add(Box<AppAdd>),
    Get(Target),
    List,
    Patch(AppPatch),
    Erase(Target),
}
#[derive(Args)]
pub(super) struct AppAdd {
    /// app.yaml / app.json, or a bundle URL with --domain.
    source: String,
    #[arg(long)]
    domain: Option<String>,
    #[arg(long)]
    alias: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    replicas: Option<u32>,
    #[arg(long, env = "WAFL_TENANT")]
    tenant: Option<String>,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long)]
    port: Option<u32>,
    #[arg(long, value_parser = ["http", "h2c"])]
    protocol: Option<String>,
    #[arg(long)]
    digest: Option<String>,
    #[arg(long, value_name = "KEY=VALUE")]
    env: Vec<String>,
}
#[derive(Args)]
pub(super) struct AppPatch {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
    #[arg(long)]
    replicas: Option<u32>,
    #[arg(long)]
    manifest: Option<PathBuf>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    expect_revision: Option<u64>,
    /// Replace the autoscaling policy using a YAML/JSON Scaling message.
    #[arg(long, conflicts_with = "fixed")]
    scaling: Option<PathBuf>,
    /// Disable autoscaling; defaults to zero replicas unless --replicas is set.
    #[arg(long)]
    fixed: bool,
    /// Replace secret file bindings from a YAML/JSON sequence; [] clears them.
    #[arg(long, conflicts_with_all = ["replicas", "manifest", "name", "expect_revision", "scaling", "fixed"])]
    secrets: Option<PathBuf>,
}
#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AppFile {
    alias: Option<String>,
    name: Option<String>,
    domain: Option<String>,
    tenant: Option<String>,
    replicas: Option<u32>,
    manifest: Option<serde_json::Value>,
    scaling: Option<serde_json::Value>,
    secrets: Option<serde_json::Value>,
}
fn fields(value: &serde_json::Value, allowed: &[&str], context: &str) -> Result<()> {
    let object = value
        .as_object()
        .with_context(|| format!("{context} must be a mapping"))?;
    for key in object.keys() {
        ensure!(
            allowed.contains(&key.as_str()),
            "unknown {context} field: {key}"
        );
    }
    Ok(())
}
fn manifest_value(value: serde_json::Value) -> Result<Manifest> {
    fields(
        &value,
        &[
            "runtime",
            "bundle",
            "port",
            "protocol",
            "abi",
            "idle_ms",
            "idleMs",
            "startup_ms",
            "startupMs",
            "memory_bytes",
            "memoryBytes",
            "max_active",
            "maxActive",
            "env",
            "bindings",
        ],
        "manifest",
    )?;
    if let Some(bundle) = value.get("bundle") {
        fields(bundle, &["url", "digest"], "bundle")?;
    }
    if let Some(bindings) = value.get("bindings") {
        for b in bindings.as_array().context("bindings must be a sequence")? {
            fields(b, &["port", "target"], "binding")?;
        }
    }
    Ok(serde_json::from_value(value)?)
}
fn scaling_value(value: serde_json::Value) -> Result<Scaling> {
    fields(
        &value,
        &[
            "min", "max", "target", "up_ms", "upMs", "down_ms", "downMs", "idle_ms", "idleMs",
            "waiting", "wait_ms", "waitMs",
        ],
        "scaling",
    )?;
    Ok(serde_json::from_value(value)?)
}
fn document<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 1024 * 1024,
        "configuration file exceeds 1 MiB"
    );
    Ok(serde_saphyr::from_str(std::str::from_utf8(&bytes)?)?)
}
fn secrets_value(value: serde_json::Value) -> Result<Vec<Secret>> {
    for binding in value.as_array().context("secrets must be a sequence")? {
        fields(binding, &["env", "file"], "secret")?;
        ensure!(
            binding
                .get("env")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty())
                && binding
                    .get("file")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty()),
            "secret requires env and file strings"
        );
    }
    Ok(serde_json::from_value(value)?)
}
// An empty repeated Patch field has no presence on the protobuf wire. Use
// generated Apply's explicit list assignment to clear it instead.
fn clear_secrets(reference: &str) -> Result<AppApplyRequest> {
    Ok(serde_json::from_value(
        serde_json::json!({"ref":app_ref(reference)?,"patch":{"messageType":"wafl.App","delta":{"entries":[{
            "targets":{"selectors":[{"key":{"field":{"name":"secrets"}}}]},
            "assign":{"value":{"l":{}}}
        }]}}}),
    )?)
}
impl AppAdd {
    fn request(self) -> Result<AppAddRequest> {
        let bundle = ["http:", "https:", "file:", "oras:"]
            .iter()
            .any(|p| self.source.starts_with(p));
        let file = if bundle {
            AppFile::default()
        } else {
            document::<AppFile>(std::path::Path::new(&self.source))?
        };
        let mut manifest = if let Some(path) = self.manifest {
            manifest_value(document(&path)?)?
        } else {
            file.manifest
                .map(manifest_value)
                .transpose()?
                .unwrap_or_default()
        };
        if bundle {
            manifest.bundle.get_or_insert_default().kind = Some(asset::Kind::Url(self.source));
        }
        if let Some(port) = self.port {
            manifest.port = port;
        } else if manifest.port == 0 {
            manifest.port = 8080;
        }
        if let Some(protocol) = self.protocol {
            manifest.protocol = protocol;
        }
        if let Some(digest) = self.digest {
            manifest.bundle.get_or_insert_default().digest = digest;
        }
        for pair in self.env {
            let (k, v) = pair.split_once('=').context("--env requires KEY=VALUE")?;
            manifest.env.insert(k.into(), v.into());
        }
        let tenant = self
            .tenant
            .or(file.tenant)
            .unwrap_or_else(|| "00000000-0000-8000-8001-000000000001".into());
        let tenant = paydirt::parse_id(&tenant)
            .filter(|id| paydirt::has_domain(id, 1))
            .context("tenant must be a domain-1 UUID")?;
        let replicas = self
            .replicas
            .or(file.replicas)
            .unwrap_or(u32::from(file.scaling.is_none()));
        ensure!(
            file.scaling.is_none() || replicas == 0,
            "use scaling or replicas, not both"
        );
        Ok(AppAddRequest {
            tenant: Some(TenantRef {
                key: Some(tenant_ref::Key::Id(tenant.to_vec())),
                ..Default::default()
            })
            .into(),
            alias: self
                .alias
                .or(file.alias)
                .map(|s| s.trim_start_matches('@').to_owned()),
            name: self.name.or(file.name).unwrap_or_default(),
            domain: self
                .domain
                .or(file.domain)
                .context("domain is required in app.yaml or --domain")?,
            manifest: Some(manifest).into(),
            replicas,
            scaling: file.scaling.map(scaling_value).transpose()?.into(),
            secrets: file
                .secrets
                .map(secrets_value)
                .transpose()?
                .unwrap_or_default(),
            ..Default::default()
        })
    }
}
#[derive(Subcommand)]
pub(super) enum PlacementCommand {
    Get {
        id: String,
    },
    List {
        #[arg(long)]
        app: Option<String>,
    },
    Release {
        id: String,
        #[arg(long, required = true)]
        withdrawn: bool,
    },
}
#[derive(Subcommand)]
pub(super) enum TrafficCommand {
    List {
        #[arg(long)]
        app: Option<String>,
    },
    /// Recover a dead observer after its process and every connection have stopped.
    Close {
        id: String,
        #[arg(long, required = true)]
        withdrawn: bool,
    },
}
pub(super) async fn traffic(
    action: TrafficCommand,
    config: ClientConfig,
    json: bool,
) -> Result<()> {
    let client = TrafficServiceClient::new(HttpClient::plaintext(), config);
    let display = |t: &Traffic| -> Result<()> {
        if json {
            println!("{}", serde_json::to_string(t)?);
        } else {
            println!(
                "{}\tsession={}\tactive={}\tclosed={}\tdraining={}\tdate_seen={}",
                paydirt::format_id(&t.id).context("invalid Traffic ID")?,
                t.session,
                t.active,
                t.closed,
                t.draining,
                t.date_seen
            );
        }
        Ok(())
    };
    match action {
        TrafficCommand::Close { id, withdrawn } => {
            let id = paydirt::parse_id(&id)
                .filter(|id| paydirt::has_domain(id, 13))
                .context("expected a domain-13 Traffic UUID")?;
            display(
                &client
                    .close(TrafficCloseRequest {
                        r#ref: Some(TrafficRef {
                            key: Some(traffic_ref::Key::Id(id.to_vec())),
                            ..Default::default()
                        })
                        .into(),
                        withdrawn,
                        ..Default::default()
                    })
                    .await?
                    .into_owned(),
            )?;
        }
        TrafficCommand::List { app } => {
            let mut request = TrafficListRequest {
                filters: app
                    .map(|a| -> Result<_> {
                        Ok(vec![TrafficFilter {
                            app: Some(app_ref(&a)?).into(),
                            ..Default::default()
                        }])
                    })
                    .transpose()?
                    .unwrap_or_default(),
                ..Default::default()
            };
            loop {
                let page = client.list(request.clone()).await?.into_owned();
                for t in &page.items {
                    display(t)?;
                }
                if page.next.is_empty() {
                    break;
                }
                request.after = page.next;
            }
        }
    }
    Ok(())
}
fn app_ref(s: &str) -> Result<AppRef> {
    Ok(AppRef {
        key: Some(if let Some(alias) = s.strip_prefix('@') {
            app_ref::Key::Alias(alias.into())
        } else {
            app_ref::Key::Id(
                paydirt::parse_id(s)
                    .filter(|id| paydirt::has_domain(id, 10))
                    .context("expected a domain-10 App UUID or @alias")?
                    .to_vec(),
            )
        }),
        ..Default::default()
    })
}
fn placement_ref(s: &str) -> Result<PlacementRef> {
    Ok(PlacementRef {
        key: Some(placement_ref::Key::Id(
            paydirt::parse_id(s)
                .filter(|id| paydirt::has_domain(id, 11))
                .context("expected a domain-11 Placement UUID")?
                .to_vec(),
        )),
        ..Default::default()
    })
}
fn print_app(a: &App, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(a)?);
    } else {
        println!(
            "{}\t@{}\t{}\treplicas={}\trevision={}",
            paydirt::format_id(&a.id).context("invalid app ID")?,
            a.alias,
            a.domain,
            if a.scaling.as_option().is_some() {
                a.state.desired
            } else {
                a.replicas
            },
            a.revision
        );
    }
    Ok(())
}
fn print_placement(p: &Placement, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(p)?);
    } else {
        println!(
            "{}\t{:?}\t{}\t{}\t{}",
            paydirt::format_id(&p.id).context("invalid placement ID")?,
            p.phase,
            p.runner,
            p.execution.endpoint.address,
            p.error
        );
    }
    Ok(())
}
pub(super) async fn app(action: AppCommand, config: ClientConfig, json: bool) -> Result<()> {
    let c = AppServiceClient::new(HttpClient::plaintext(), config);
    match action {
        AppCommand::Add(a) => {
            print_app(&c.add(a.request()?).await?.into_owned(), json)?;
        }
        AppCommand::Get(a) => print_app(
            &c.get(AppGetRequest {
                r#ref: Some(app_ref(&a.reference)?).into(),
                ..Default::default()
            })
            .await?
            .into_owned(),
            json,
        )?,
        AppCommand::Patch(a) => {
            if let Some(path) = a.secrets {
                let secrets = secrets_value(document(&path)?)?;
                let updated = if secrets.is_empty() {
                    c.apply(clear_secrets(&a.reference)?).await?.into_owned()
                } else {
                    c.patch(AppPatchRequest {
                        r#ref: Some(app_ref(&a.reference)?).into(),
                        secrets,
                        ..Default::default()
                    })
                    .await?
                    .into_owned()
                };
                print_app(&updated, json)?;
                return Ok(());
            }
            let manifest = a
                .manifest
                .map(|path| -> Result<Manifest> { manifest_value(document(&path)?) })
                .transpose()?;
            print_app(
                &c.patch(AppPatchRequest {
                    r#ref: Some(app_ref(&a.reference)?).into(),
                    replicas: a.replicas.or(if a.fixed || a.scaling.is_some() {
                        Some(0)
                    } else {
                        None
                    }),
                    name: a.name,
                    revision: a.expect_revision,
                    manifest: manifest.into(),
                    scaling: a
                        .scaling
                        .map(|p| scaling_value(document(&p)?))
                        .transpose()?
                        .into(),
                    scaling_null: if a.fixed { Some(true) } else { None },
                    ..Default::default()
                })
                .await?
                .into_owned(),
                json,
            )?;
        }
        AppCommand::Erase(a) => {
            let r = c.erase(app_ref(&a.reference)?).await?.into_owned();
            if json {
                println!("{}", serde_json::to_string(&r)?);
            } else {
                println!("{}", r.erased.unwrap_or(false));
            }
        }
        AppCommand::List => {
            let mut req = AppListRequest::default();
            loop {
                let page = c.list(req.clone()).await?.into_owned();
                for a in &page.items {
                    print_app(a, json)?;
                }
                if page.next.is_empty() {
                    break;
                }
                req.after = page.next;
            }
        }
    }
    Ok(())
}
pub(super) async fn placement(
    action: PlacementCommand,
    config: ClientConfig,
    json: bool,
) -> Result<()> {
    let c = PlacementServiceClient::new(HttpClient::plaintext(), config);
    match action {
        PlacementCommand::Get { id } => print_placement(
            &c.get(PlacementGetRequest {
                r#ref: Some(placement_ref(&id)?).into(),
                ..Default::default()
            })
            .await?
            .into_owned(),
            json,
        )?,
        PlacementCommand::Release { id, withdrawn } => print_placement(
            &c.release(PlacementReleaseRequest {
                r#ref: Some(placement_ref(&id)?).into(),
                withdrawn,
                ..Default::default()
            })
            .await?
            .into_owned(),
            json,
        )?,
        PlacementCommand::List { app } => {
            let mut req = PlacementListRequest {
                filters: app
                    .map(|r| -> Result<_> {
                        Ok(vec![PlacementFilter {
                            app: Some(app_ref(&r)?).into(),
                            ..Default::default()
                        }])
                    })
                    .transpose()?
                    .unwrap_or_default(),
                ..Default::default()
            };
            loop {
                let page = c.list(req.clone()).await?.into_owned();
                for p in &page.items {
                    print_placement(p, json)?;
                }
                if page.next.is_empty() {
                    break;
                }
                req.after = page.next;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(file: &std::path::Path, flags: &[&str]) -> Result<AppAddRequest> {
        let mut args = vec!["wafl", "app", "add", file.to_str().unwrap()];
        args.extend(flags);
        let cli = Cli::try_parse_from(args)?;
        let Command::App {
            action: AppCommand::Add(add),
        } = cli.command
        else {
            panic!("App.Add")
        };
        add.request()
    }
    #[test]
    fn yaml_has_defaults_explicit_overrides_and_strict_nested_fields() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("app.yaml");
        std::fs::write(&file,"alias: hello\ndomain: hello.example\nmanifest:\n  bundle: {url: 'https://example.com/app.tar.gz'}\n  memory_bytes: 33554432\n  env: {COUNT: '4'}\nscaling: {min: 0, max: 3}\n").unwrap();
        let req = request(&file, &["--domain", "override.example", "--env", "COUNT=5"]).unwrap();
        assert_eq!(req.domain, "override.example");
        assert_eq!(req.manifest.env["COUNT"], "5");
        assert_eq!(req.manifest.port, 8080);
        assert_eq!(req.manifest.memory_bytes, 33554432);
        assert_eq!(req.replicas, 0);
        assert_eq!(req.scaling.max, 3);
        assert!(request(&file, &["--replicas", "1"]).is_err());
        for bad in [
            "domain: test.example\nscailing: {}",
            "domain: test.example\nmanifest: {memory_byte: 1}",
            "domain: test.example\nscaling: {targets: 1}",
            "domain: test.example\nmanifest: {env: {BOOL: true}}",
            "domain: test.example\nmanifest: {bundle: {uri: 'file:/x'}}",
        ] {
            std::fs::write(&file, bad).unwrap();
            assert!(request(&file, &[]).is_err(), "accepted {bad}");
        }
    }
    #[test]
    fn yaml_secret_sources_are_references_and_clear_is_explicit() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("app.yaml");
        std::fs::write(&file,"domain: test.example\nmanifest: {bundle: {url: 'file:/app.tar.gz'}}\nsecrets: [{env: TOKEN, file: does-not-exist-here}]\n").unwrap();
        let req = request(&file, &[]).unwrap();
        assert_eq!(req.secrets[0].env, "TOKEN");
        assert!(req.manifest.env.is_empty());
        for bad in [
            serde_json::json!([{"env":"TOKEN","value":"inline"}]),
            serde_json::json!([{"env":"TOKEN","secretKeyRef":{"name":"db","key":"url"}}]),
            serde_json::json!([{"file":"x"}]),
        ] {
            assert!(secrets_value(bad).is_err());
        }
        let patch = clear_secrets("@test").unwrap();
        let json = serde_json::to_value(patch).unwrap();
        assert!(json["patch"]["delta"]["entries"][0]["assign"]["value"]["l"].is_object());
    }
}
