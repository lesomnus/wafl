use anyhow::{Result, anyhow, ensure};
use std::path::PathBuf;
use wafl_runner::{config::Config, host};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow!("usage: wafl-runner <runner.json>"))?,
    );
    ensure!(args.next().is_none(), "usage: wafl-runner <runner.json>");
    host::serve(Config::read(&path)?).await
}
