use anyhow::{Context, Result, ensure};
use clap::{Args, Parser, Subcommand};
use connectrpc::client::{ClientConfig, HttpClient};
use std::{io::Write, net::SocketAddr, path::PathBuf};
use wafl_runner::{
    config::{Config, INSTANCE_DOMAIN, TENANT_DOMAIN, valid_alias},
    host,
    proto::wafl::*,
    services::wafl::{InstanceServiceClient, RunnerServiceClient},
};

#[path = "wafl/apps.rs"]
mod apps;
#[path = "wafl/reservations.rs"]
mod reservations;

#[derive(Parser)]
#[command(version, about = "Run WASM applications, controllers and runners")]
struct Cli {
    #[arg(
        long,
        global = true,
        env = "WAFL_ENDPOINT",
        default_value = "http://127.0.0.1:8081"
    )]
    endpoint: String,
    /// Print protobuf JSON responses (one record per line for logs).
    #[arg(long, global = true)]
    json: bool,
    /// Reject a mutation if the runner process has restarted.
    #[arg(long, global = true)]
    session: Option<String>,
    /// Require this lifecycle revision (requires --session).
    #[arg(long, global = true, requires = "session")]
    revision: Option<u64>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Inspect autoscaling observers and recover an explicitly stopped observer.
    Traffic {
        #[command(subcommand)]
        action: apps::TrafficCommand,
    },
    /// Manage controller-owned applications.
    App {
        #[command(subcommand)]
        action: apps::AppCommand,
    },
    /// Inspect placements and acknowledge completed gateway withdrawal.
    Placement {
        #[command(subcommand)]
        action: apps::PlacementCommand,
    },
    /// Inspect durable direct-port reservations and reclaim orphan allocations.
    Reservation {
        #[command(subcommand)]
        action: reservations::Command,
    },
    /// Inspect the current runner session and capacity.
    Runner {
        #[command(subcommand)]
        action: RunnerCommand,
    },
    /// Run a server in the foreground until SIGINT or SIGTERM.
    Serve {
        #[command(subcommand)]
        role: Serve,
    },
    /// Manage instances using the generated InstanceService client.
    Instance {
        #[command(subcommand)]
        action: InstanceCommand,
    },
}
#[derive(Subcommand)]
enum RunnerCommand {
    Get,
}
#[derive(Subcommand)]
enum Serve {
    Runner(RunnerArgs),
    Controller(ControllerArgs),
}
#[derive(Args)]
struct ControllerArgs {
    #[arg(long)]
    config: PathBuf,
    #[arg(long)]
    listen: Option<SocketAddr>,
    /// Adopt changed deployment settings after stopping old controller replicas.
    #[arg(long)]
    configure: bool,
}
#[derive(Args)]
struct RunnerArgs {
    /// Shared bundle cache. Default: $XDG_CACHE_HOME/wafl or $HOME/.cache/wafl.
    #[arg(value_name = "DIRECTORY")]
    path: Option<PathBuf>,
    #[arg(long, conflicts_with = "path")]
    directory: Option<PathBuf>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    listen: Option<SocketAddr>,
    #[arg(long)]
    control: Option<SocketAddr>,
}
#[derive(Subcommand)]
enum InstanceCommand {
    Add(Box<Add>),
    Get(Target),
    List,
    Start(Target),
    Stop(Target),
    Drain(Drain),
    /// Finish a direct drain after every gateway has removed routes and drained connections.
    Release(Release),
    Erase(Target),
    Logs(Logs),
}
#[derive(Args)]
struct Target {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
}
#[derive(Args)]
struct Drain {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
    #[arg(long, default_value_t = 0)]
    timeout_ms: u64,
    #[arg(long)]
    force: bool,
}
#[derive(Args)]
struct Release {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
    /// Exact generation whose gateway routes and connections have been drained.
    #[arg(long)]
    generation: u64,
}
#[derive(Args)]
struct Add {
    /// Caller-chosen domain-7 UUID, for Get/retry after an uncertain response.
    #[arg(long)]
    id: Option<String>,
    /// Bundle URL, resolved by the runner (http, https, or file).
    url: String,
    #[arg(long)]
    alias: Option<String>,
    #[arg(long, default_value = "")]
    name: String,
    /// Tenant metadata; the default is a fixed local-development tenant.
    #[arg(
        long,
        env = "WAFL_TENANT",
        default_value = "00000000-0000-8000-8001-000000000001"
    )]
    tenant: String,
    /// Base Manifest in protobuf JSON. URL and explicit flags override its fields.
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Guest listening port; defaults to 8080 without a base manifest.
    #[arg(long)]
    port: Option<u32>,
    #[arg(long, value_parser = ["http", "h2c"])]
    protocol: Option<String>,
    #[arg(long)]
    idle_ms: Option<u64>,
    #[arg(long)]
    startup_ms: Option<u64>,
    #[arg(long)]
    memory_bytes: Option<u64>,
    #[arg(long)]
    max_active: Option<u32>,
    #[arg(long, value_name = "KEY=VALUE")]
    env: Vec<String>,
    #[arg(long, value_name = "PORT=IP:PORT")]
    binding: Vec<String>,
    /// Expected SHA-256 of the compressed bundle, including sha256: prefix.
    #[arg(long)]
    digest: Option<String>,
}
#[derive(Args)]
struct Logs {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
    #[arg(short = 'f', long)]
    follow: bool,
    /// Resume beyond a log sequence. Expired cursors return an error.
    #[arg(long, default_value_t = 0)]
    cursor: u64,
    /// Initial retained records, not lines. Zero means all; maximum 4096.
    #[arg(long, default_value_t = 0)]
    limit: u32,
}

fn reference(value: &str) -> Result<InstanceRef> {
    let key = if let Some(alias) = value.strip_prefix('@') {
        ensure!(
            valid_alias(alias),
            "invalid alias: use @ followed by 1..63 lowercase letters, digits or hyphens"
        );
        instance_ref::Key::Alias(alias.into())
    } else {
        let id = paydirt::parse_id(value)
            .filter(|id| paydirt::has_domain(id, INSTANCE_DOMAIN))
            .context("expected an Instance UUID or @alias")?;
        instance_ref::Key::Id(id.to_vec())
    };
    Ok(InstanceRef {
        key: Some(key),
        ..Default::default()
    })
}
fn print_instance(instance: &Instance, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(instance)?);
    } else {
        print!(
            "{}\t@{}",
            paydirt::format_id(&instance.id).context("invalid instance ID in response")?,
            instance.alias
        );
        if let Some(endpoint) = instance.endpoint.as_option() {
            print!("\t{}", endpoint.address);
        }
        println!();
    }
    Ok(())
}
impl Add {
    fn request(self) -> Result<InstanceAddRequest> {
        let tenant = paydirt::parse_id(&self.tenant)
            .filter(|id| paydirt::has_domain(id, TENANT_DOMAIN))
            .context("tenant must be a domain-1 UUID")?;
        let mut manifest = match self.manifest {
            Some(path) => serde_json::from_slice::<Manifest>(&std::fs::read(path)?)?,
            None => Manifest {
                port: 8080,
                ..Default::default()
            },
        };
        manifest.bundle.get_or_insert_default().kind = Some(asset::Kind::Url(self.url));
        if let Some(digest) = self.digest {
            manifest.bundle.get_or_insert_default().digest = digest;
        }
        if let Some(port) = self.port {
            manifest.port = port;
        }
        if let Some(protocol) = self.protocol {
            manifest.protocol = protocol;
        }
        if let Some(value) = self.idle_ms {
            manifest.idle_ms = value;
        }
        if let Some(value) = self.startup_ms {
            manifest.startup_ms = value;
        }
        if let Some(value) = self.memory_bytes {
            manifest.memory_bytes = value;
        }
        if let Some(value) = self.max_active {
            manifest.max_active = value;
        }
        for entry in self.env {
            let (key, value) = entry.split_once('=').context("--env requires KEY=VALUE")?;
            manifest.env.insert(key.into(), value.into());
        }
        for entry in self.binding {
            let (port, target) = entry
                .split_once('=')
                .context("--binding requires PORT=IP:PORT")?;
            manifest.bindings.push(Binding {
                port: port.parse()?,
                target: target.into(),
                ..Default::default()
            });
        }
        let alias = self.alias.unwrap_or_default();
        let alias = alias.strip_prefix('@').unwrap_or(&alias).to_owned();
        Ok(InstanceAddRequest {
            id: self
                .id
                .map(|value| {
                    paydirt::parse_id(&value)
                        .filter(|id| paydirt::has_domain(id, INSTANCE_DOMAIN))
                        .map(|id| id.to_vec())
                        .context("id must be a domain-7 UUID")
                })
                .transpose()?,
            tenant: Some(TenantRef {
                key: Some(tenant_ref::Key::Id(tenant.to_vec())),
                ..Default::default()
            })
            .into(),
            alias: if alias.is_empty() { None } else { Some(alias) },
            name: self.name,
            manifest: Some(manifest).into(),
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let action = match cli.command {
        Command::Serve {
            role: Serve::Controller(args),
        } => {
            let mut config = wafl_runner::controller::Config::read(&args.config)?;
            if let Some(listen) = args.listen {
                config.listen = listen;
            }
            return wafl_runner::controller::serve(config, args.configure).await;
        }
        Command::App { action } => {
            ensure!(
                cli.session.is_none() && cli.revision.is_none(),
                "controller commands use --expect-revision, not runner session/revision flags"
            );
            return apps::app(action, client_config(&cli.endpoint, None, None)?, cli.json).await;
        }
        Command::Placement { action } => {
            ensure!(
                cli.session.is_none() && cli.revision.is_none(),
                "placement identity includes its immutable runner session"
            );
            return apps::placement(action, client_config(&cli.endpoint, None, None)?, cli.json)
                .await;
        }
        Command::Reservation { action } => {
            return reservations::run(
                action,
                client_config(&cli.endpoint, cli.session.as_deref(), cli.revision)?,
                cli.json,
            )
            .await;
        }
        Command::Serve {
            role: Serve::Runner(args),
        } => {
            let mut config = match args.config {
                Some(path) => Config::read(&path)?,
                None => Config {
                    listen: "127.0.0.1:8080".parse()?,
                    control: "127.0.0.1:8081".parse()?,
                    directory: None,
                    metrics: None,
                    bundle: Default::default(),
                    cache: Default::default(),
                    capacity: Default::default(),
                    direct: None,
                },
            };
            if let Some(path) = args.path.or(args.directory) {
                config.directory = Some(path);
            }
            if let Some(address) = args.listen {
                config.listen = address;
            }
            if let Some(address) = args.control {
                config.control = address;
            }
            eprintln!("bundle directory: {}", config.directory()?.display());
            return host::serve(config).await;
        }
        Command::Runner {
            action: RunnerCommand::Get,
        } => {
            let client = RunnerServiceClient::new(
                HttpClient::plaintext(),
                client_config(&cli.endpoint, None, None)?,
            );
            let value = client
                .get(RunnerGetRequest {
                    r#ref: Some(RunnerRef {
                        key: Some(runner_ref::Key::Alias("self".into())),
                        ..Default::default()
                    })
                    .into(),
                    ..Default::default()
                })
                .await?
                .into_owned();
            if cli.json {
                println!("{}", serde_json::to_string(&value)?);
            } else {
                println!(
                    "{}\tingress={}\tcontrol={}\taccepting={}",
                    paydirt::format_id(&value.id).context("invalid runner ID")?,
                    value.ingress,
                    value.control,
                    value.accepting
                );
            }
            return Ok(());
        }
        Command::Traffic { action } => {
            apps::traffic(action, client_config(&cli.endpoint, None, None)?, cli.json).await?;
            return Ok(());
        }
        Command::Instance { action } => action,
    };
    let client = InstanceServiceClient::new(
        HttpClient::plaintext(),
        client_config(&cli.endpoint, cli.session.as_deref(), cli.revision)?,
    );
    match action {
        InstanceCommand::Add(args) => print_instance(
            &client.add((*args).request()?).await?.into_owned(),
            cli.json,
        )?,
        InstanceCommand::Get(args) => print_instance(
            &client
                .get(InstanceGetRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    ..Default::default()
                })
                .await?
                .into_owned(),
            cli.json,
        )?,
        InstanceCommand::Start(args) => print_instance(
            &client
                .start(InstanceStartRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    ..Default::default()
                })
                .await?
                .into_owned(),
            cli.json,
        )?,
        InstanceCommand::Stop(args) => print_instance(
            &client
                .stop(InstanceStopRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    ..Default::default()
                })
                .await?
                .into_owned(),
            cli.json,
        )?,
        InstanceCommand::Drain(args) => print_instance(
            &client
                .drain(InstanceDrainRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    timeout_ms: args.timeout_ms,
                    force: args.force,
                    ..Default::default()
                })
                .await?
                .into_owned(),
            cli.json,
        )?,
        InstanceCommand::Release(args) => print_instance(
            &client
                .release(InstanceReleaseRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    generation: args.generation,
                    ..Default::default()
                })
                .await?
                .into_owned(),
            cli.json,
        )?,
        InstanceCommand::Erase(args) => {
            let result = client
                .erase(reference(&args.reference)?)
                .await?
                .into_owned();
            if cli.json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("{}", result.erased.unwrap_or(false));
            }
        }
        InstanceCommand::List => {
            let mut request = InstanceListRequest::default();
            loop {
                let page = client.list(request.clone()).await?.into_owned();
                for instance in &page.items {
                    print_instance(instance, cli.json)?;
                }
                if page.next.is_empty() {
                    break;
                }
                request.after = page.next;
            }
        }
        InstanceCommand::Logs(args) => {
            let mut stream = client
                .tail(InstanceTailRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    follow: args.follow,
                    cursor: args.cursor,
                    limit: args.limit,
                    ..Default::default()
                })
                .await?;
            loop {
                let message = tokio::select! { message = stream.message() => message?, _ = tokio::signal::ctrl_c() => break };
                let Some(message) = message else {
                    break;
                };
                let entry = message.to_owned_message();
                if cli.json {
                    println!("{}", serde_json::to_string(&entry)?);
                } else if entry.channel == Channel::Stderr {
                    let mut out = std::io::stderr().lock();
                    out.write_all(&entry.data)?;
                    out.flush()?;
                } else {
                    let mut out = std::io::stdout().lock();
                    out.write_all(&entry.data)?;
                    out.flush()?;
                }
            }
        }
    }
    Ok(())
}

fn client_config(
    endpoint: &str,
    session: Option<&str>,
    revision: Option<u64>,
) -> Result<ClientConfig> {
    let uri: http::Uri = endpoint.parse().context("invalid endpoint")?;
    ensure!(
        uri.scheme_str() == Some("http") && uri.authority().is_some(),
        "runner endpoint must be an http:// address"
    );
    let mut headers = http::HeaderMap::new();
    if let Some(session) = session {
        ensure!(
            paydirt::parse_id(session).is_some_and(|id| paydirt::has_domain(&id, 8)),
            "session must be a domain-8 Runner UUID"
        );
        headers.insert("wafl-session", session.parse()?);
    }
    if let Some(revision) = revision {
        headers.insert("wafl-revision", revision.to_string().parse()?);
    }
    Ok(ClientConfig::new(uri).with_default_headers(headers))
}
