//! WAFL's Pingora data plane. Controllers publish candidates; persisted Traffic
//! gates authorize forwarding and fence reuse until every stream has finished.
mod control;
mod http;
mod state;
mod tcp;
mod traffic;

use anyhow::{Result, ensure};
use async_trait::async_trait;
use clap::Args;
use pingora::{
    apps::HttpServerOptions,
    proxy::{http_proxy, http_proxy_service},
    server::{Server, ShutdownWatch, configuration::ServerConf},
    services::{
        background::{BackgroundService, background_service},
        listening::Service,
    },
};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Args, Clone, Debug)]
pub struct Options {
    /// Controller management origin; application bytes never pass through it.
    #[arg(long, env = "WAFL_CONTROLLER", default_value = "http://127.0.0.1:8082")]
    pub controller: String,
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub listen: SocketAddr,
    #[arg(long, default_value = "0.0.0.0:8084")]
    pub health: SocketAddr,
    /// Bind the platform TCP port pool (10000..10015) on this address.
    #[arg(long, default_value = "0.0.0.0")]
    pub tcp_ip: IpAddr,
    /// Disable general TCP listeners, for an HTTP-only proxy.
    #[arg(long)]
    pub http_only: bool,
    #[arg(long,requires_all=["tls_cert","tls_key"])]
    pub tls_listen: Option<SocketAddr>,
    #[arg(long, requires = "tls_listen")]
    pub tls_cert: Option<PathBuf>,
    #[arg(long, requires = "tls_listen")]
    pub tls_key: Option<PathBuf>,
    /// Maximum admitted HTTP requests plus waiting/active TCP connections.
    #[arg(long, default_value_t = 1024)]
    pub capacity: usize,
    /// Worker threads for each HTTP and TCP service.
    #[arg(long, default_value_t = 2)]
    pub threads: usize,
    /// Stop accepting new work, then allow existing streams to finish.
    #[arg(long, default_value_t = 30)]
    pub drain_seconds: u64,
}
struct Updates {
    state: Arc<state::State>,
    grace: u64,
}
#[async_trait]
impl BackgroundService for Updates {
    async fn start(&self, mut shutdown: ShutdownWatch) {
        loop {
            if *shutdown.borrow() {
                break;
            }
            if let Err(error) = self.state.refresh().await {
                log::warn!("route refresh: {error}");
            }
            tokio::select! {_=shutdown.changed()=>break,_=tokio::time::sleep(Duration::from_millis(250))=>{}}
        }
        self.state.stop();
        let end = Instant::now() + Duration::from_secs(self.grace + 3);
        while !self.state.drained() && Instant::now() < end {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if !self.state.drained() {
            log::error!(
                "proxy stopped with unacknowledged observers; retain reservations until explicit Traffic recovery"
            );
        }
    }
}

#[cfg(unix)]
struct Signals;
#[cfg(unix)]
#[async_trait]
impl pingora::server::ShutdownSignalWatch for Signals {
    async fn recv(&self) -> pingora::server::ShutdownSignal {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut interrupt = signal(SignalKind::interrupt()).expect("SIGINT handler");
        tokio::select! {_=term.recv()=>{},_=interrupt.recv()=>{}}
        pingora::server::ShutdownSignal::GracefulTerminate
    }
}

pub fn serve(options: Options) -> Result<()> {
    ensure!(
        (1..=128).contains(&options.threads),
        "threads must be between 1 and 128"
    );
    ensure!(
        (1..=1_000_000).contains(&options.capacity),
        "capacity must be between 1 and 1000000"
    );
    ensure!(
        (1..=3600).contains(&options.drain_seconds),
        "drain-seconds must be between 1 and 3600"
    );
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    let state = state::State::new(
        control::Control::new(&options.controller)?,
        options.capacity,
    );
    let conf = ServerConf {
        threads: options.threads,
        // Pingora counts the initial attempt in max_retries. One forbids replay.
        max_retries: 1,
        grace_period_seconds: Some(options.drain_seconds + 4),
        graceful_shutdown_timeout_seconds: Some(1),
        ..Default::default()
    };
    let mut server = Server::new_with_opt_and_conf(None, conf);
    server.bootstrap();
    let mut engine = http_proxy(
        &server.configuration,
        http::Http {
            state: state.clone(),
            health: false,
        },
    );
    let mut http_options = HttpServerOptions::default();
    http_options.h2c = true;
    engine.server_options = Some(http_options);
    let mut http = Service::new(
        "WAFL HTTP".into(),
        http::Transport {
            inner: Arc::new(engine),
            grace: options.drain_seconds,
        },
    );
    http.add_tcp(&options.listen.to_string());
    if let Some(address) = options.tls_listen {
        let cert = options
            .tls_cert
            .as_ref()
            .unwrap()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid certificate path"))?;
        let key = options
            .tls_key
            .as_ref()
            .unwrap()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid key path"))?;
        let mut tls = pingora::listeners::tls::TlsSettings::intermediate(cert, key)?;
        tls.enable_h2();
        http.add_tls_with_settings(&address.to_string(), None, tls);
    }
    server.add_service(http);
    let mut health = http_proxy_service(
        &server.configuration,
        http::Http {
            state: state.clone(),
            health: true,
        },
    );
    health.threads = Some(1);
    health.add_tcp(&options.health.to_string());
    server.add_service(health);
    if !options.http_only {
        let mut tcp = Service::new(
            "WAFL TCP".into(),
            tcp::Tcp {
                state: state.clone(),
                grace: options.drain_seconds,
            },
        );
        for port in 10000..=10015 {
            tcp.add_tcp(&SocketAddr::new(options.tcp_ip, port).to_string());
        }
        server.add_service(tcp);
    }
    server.add_service(background_service(
        "WAFL routes and drain",
        Updates {
            state,
            grace: options.drain_seconds,
        },
    ));
    #[cfg(unix)]
    server.run(pingora::server::RunArgs {
        shutdown_signal: Box::new(Signals),
    });
    #[cfg(not(unix))]
    server.run(Default::default());
    Ok(())
}
