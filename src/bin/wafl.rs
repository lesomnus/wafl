use anyhow::{Context, Result, bail, ensure};
use clap::{Args, Parser, Subcommand};
use connectrpc::client::{ClientConfig, HttpClient};
use std::{io::Write, net::SocketAddr, path::PathBuf};
use wafl_runner::{
    config::{Config, INSTANCE_DOMAIN, TENANT_DOMAIN, valid_alias},
    host,
    proto::wafl::*,
    services::wafl::InstanceServiceClient,
};

#[derive(Parser)]
#[command(version, about = "Run WASM socket servers and manage runner instances")]
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
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
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
enum Serve {
    Runner(RunnerArgs),
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
    Erase(Target),
    Logs(Logs),
}
#[derive(Args)]
struct Target {
    #[arg(value_name = "ID|@ALIAS")]
    reference: String,
}
#[derive(Args)]
struct Add {
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
        println!(
            "{}\t@{}",
            paydirt::format_id(&instance.id).context("invalid instance ID in response")?,
            instance.alias
        );
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
        Command::Instance { action } => action,
    };
    let uri: http::Uri = cli.endpoint.parse().context("invalid endpoint")?;
    if uri.scheme_str() != Some("http") || uri.authority().is_none() {
        bail!("runner endpoint must be an http:// address");
    }
    let client = InstanceServiceClient::new(HttpClient::plaintext(), ClientConfig::new(uri));
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
