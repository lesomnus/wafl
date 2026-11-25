use crate::{config::Config, control, proxy, runner::Runner, telemetry::Telemetry};
use anyhow::{Result, anyhow};
use paydirt::spin::{Lease, Placement, Workers};
use std::time::Duration;

pub async fn serve(config: Config) -> Result<()> {
    config.validate()?;
    let runner = Runner::new(&config).await?;
    let ingress = if runner.direct() {
        None
    } else {
        Some(tokio::net::TcpListener::bind(config.listen).await?)
    };
    let control = paydirt_server::Host::bind(
        config.control,
        paydirt_server::Serving::default(),
        "wafl-runner",
        control::router(runner.clone()),
    )
    .await
    .map_err(|e| anyhow!("{e}"))?;
    let control_addr = control.local_addr()?;
    let ingress_addr = ingress.as_ref().map(|s| s.local_addr()).transpose()?;
    runner.addresses(ingress_addr.unwrap_or(config.listen), control_addr);
    let mut workers = Workers::default();
    if !runner.direct() {
        let reaper = runner.clone();
        workers
            .add("wafl.reap", Placement::Everywhere, move |stop| {
                let runner = reaper.clone();
                async move {
                    let mut tick = tokio::time::interval(Duration::from_millis(100));
                    loop {
                        tokio::select! {
                            _ = stop.cancelled() => return Ok(()),
                            _ = tick.tick() => runner.reap().await,
                        }
                    }
                }
            })
            .map_err(|e| anyhow!(e))?;
    }
    let telemetry = match config.metrics {
        Some(metrics) => Some(
            Telemetry::start(
                metrics,
                runner.observe(),
                paydirt::format_id(&runner.session()).unwrap(),
            )
            .await?,
        ),
        None => None,
    };
    eprintln!(
        "wafl-runner ingress={} control={}",
        ingress_addr
            .map(|a| a.to_string())
            .unwrap_or_else(|| "direct".into()),
        control_addr
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
                        match ingress {
                            Some(ingress) => proxy::serve(ingress, traffic, stop.clone())
                                .await
                                .map_err(|e| e.to_string()),
                            None => {
                                stop.cancelled().await;
                                Ok(())
                            }
                        }
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

pub(crate) async fn shutdown() {
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
