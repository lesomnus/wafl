use super::*;
use wafl_runner::services::wafl::{AppServiceClient, PlacementServiceClient};

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
    url: String,
    #[arg(long)]
    domain: String,
    #[arg(long)]
    alias: Option<String>,
    #[arg(long, default_value = "")]
    name: String,
    #[arg(long, default_value_t = 1)]
    replicas: u32,
    #[arg(
        long,
        env = "WAFL_TENANT",
        default_value = "00000000-0000-8000-8001-000000000001"
    )]
    tenant: String,
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
            a.replicas,
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
            let mut manifest = match a.manifest {
                Some(path) => serde_json::from_slice::<Manifest>(&std::fs::read(path)?)?,
                None => Manifest {
                    port: 8080,
                    ..Default::default()
                },
            };
            manifest.bundle.get_or_insert_default().kind = Some(asset::Kind::Url(a.url));
            if let Some(port) = a.port {
                manifest.port = port;
            }
            if let Some(protocol) = a.protocol {
                manifest.protocol = protocol;
            }
            if let Some(digest) = a.digest {
                manifest.bundle.get_or_insert_default().digest = digest;
            }
            for pair in a.env {
                let (k, v) = pair.split_once('=').context("--env requires KEY=VALUE")?;
                manifest.env.insert(k.into(), v.into());
            }
            let tenant = paydirt::parse_id(&a.tenant)
                .filter(|id| paydirt::has_domain(id, 1))
                .context("tenant must be a domain-1 UUID")?;
            print_app(
                &c.add(AppAddRequest {
                    tenant: Some(TenantRef {
                        key: Some(tenant_ref::Key::Id(tenant.to_vec())),
                        ..Default::default()
                    })
                    .into(),
                    alias: a.alias.map(|s| s.trim_start_matches('@').to_owned()),
                    name: a.name,
                    domain: a.domain,
                    replicas: a.replicas,
                    manifest: Some(manifest).into(),
                    ..Default::default()
                })
                .await?
                .into_owned(),
                json,
            )?;
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
            let manifest = a
                .manifest
                .map(|path| -> Result<Manifest> {
                    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
                })
                .transpose()?;
            print_app(
                &c.patch(AppPatchRequest {
                    r#ref: Some(app_ref(&a.reference)?).into(),
                    replicas: a.replicas,
                    name: a.name,
                    revision: a.expect_revision,
                    manifest: manifest.into(),
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
