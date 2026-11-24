use anyhow::{Result, anyhow, ensure};
use paydirt::spin::{Lease, Placement, Workers};
use std::{path::PathBuf, time::Duration};
use wafl_runner::{config::Config, control, proxy, runner::Runner};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow!("usage: wafl-runner <manifest.json>"))?,
    );
    ensure!(args.next().is_none(), "usage: wafl-runner <manifest.json>");
    let config = Config::read(&path)?;
    let token = std::env::var("WAFL_TOKEN").ok().filter(|v| !v.is_empty());
    ensure!(
        config.control.ip().is_loopback() || token.is_some(),
        "WAFL_TOKEN is required for a non-loopback control listener"
    );
    let runner = Runner::new(&config).await?;
    let ingress = tokio::net::TcpListener::bind(config.listen).await?;
    let control = paydirt_server::Host::bind(
        config.control,
        paydirt_server::Serving::default(),
        "wafl-runner",
        control::router(runner.clone(), token),
    )
    .await
    .map_err(|e| anyhow!("{e}"))?;
    eprintln!(
        "wafl-runner ingress={} control={} apps={}",
        ingress.local_addr()?,
        config.control,
        config.apps.len()
    );
    let mut workers = Workers::default();
    let reaper = runner.clone();
    workers.add("wafl.reap", Placement::Everywhere, move |stop| {
        let runner = reaper.clone();
        async move {
            let mut tick = tokio::time::interval(Duration::from_millis(100));
            loop {
                tokio::select! { _ = stop.cancelled() => return Ok(()), _ = tick.tick() => runner.reap().await }
            }
        }
    }).map_err(|e| anyhow!(e))?;
    let traffic = runner.clone();
    let result = workers
        .run(Lease::Local, shutdown(), move |stop| async move {
            tokio::try_join!(
                async { control.serve(stop.clone()).await.map_err(|e| e.to_string()) },
                async {
                    proxy::serve(ingress, traffic, stop.clone())
                        .await
                        .map_err(|e| e.to_string())
                }
            )
            .map(|_| ())
        })
        .await;
    runner.shutdown().await;
    result.map_err(|e| anyhow!(e))
}

async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! { _ = term.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}
