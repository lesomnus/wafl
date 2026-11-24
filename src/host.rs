use crate::{config::Config, control, proxy, runner::Runner, telemetry::Telemetry};
use anyhow::{Result, anyhow};
use paydirt::spin::{Lease, Placement, Workers};
use std::time::Duration;

pub async fn serve(config: Config) -> Result<()> {
    config.validate()?;
    let runner = Runner::new(&config).await?;
    let ingress = tokio::net::TcpListener::bind(config.listen).await?;
    let control = paydirt_server::Host::bind(
        config.control,
        paydirt_server::Serving::default(),
        "wafl-runner",
        control::router(runner.clone()),
    )
    .await
    .map_err(|e| anyhow!("{e}"))?;
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
    let telemetry = match config.metrics {
        Some(metrics) => Some(Telemetry::start(metrics, runner.observe()).await?),
        None => None,
    };
    eprintln!(
        "wafl-runner ingress={} control={}",
        ingress.local_addr()?,
        config.control
    );
    let traffic = runner.clone();
    let draining = runner.clone();
    let result = workers
        .run(
            Lease::Local,
            async move {
                shutdown().await;
                draining.drain();
            },
            move |stop| async move {
                tokio::try_join!(
                    async { control.serve(stop.clone()).await.map_err(|e| e.to_string()) },
                    async {
                        proxy::serve(ingress, traffic, stop.clone())
                            .await
                            .map_err(|e| e.to_string())
                    }
                )
                .map(|_| ())
            },
        )
        .await;
    runner.shutdown().await;
    if let Some(telemetry) = telemetry {
        telemetry.shutdown().await;
    }
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
