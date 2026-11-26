use anyhow::{Context, Result};
use std::{ffi::OsString, path::PathBuf, process::Command};

/// App commands use the controller API. Server roles and the platform client
/// share a Go companion; runner commands remain self-contained.
pub fn dispatch() -> Result<()> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut index = 0;
    while index < args.len() {
        match args[index].to_str().unwrap_or("") {
            "--endpoint" | "--tenant" => index += 2,
            "--json" => index += 1,
            value
                if ["--endpoint=", "--tenant="]
                    .iter()
                    .any(|prefix| value.starts_with(prefix)) =>
            {
                index += 1
            }
            _ => break,
        }
    }
    let command = args.get(index).and_then(|v| v.to_str());
    let native = matches!(command, Some("app" | "traffic" | "assets"))
        || command == Some("serve")
            && matches!(
                args.get(index + 1).and_then(|v| v.to_str()),
                Some("controller" | "node")
            );
    if !native {
        return Ok(());
    }
    let executable = if let Some(path) = std::env::var_os("WAFL_KUBE_BINARY") {
        PathBuf::from(path)
    } else {
        let sibling = std::env::current_exe()?.with_file_name("wafl-kube");
        if sibling.is_file() {
            sibling
        } else {
            PathBuf::from("wafl-kube")
        }
    };
    let mut command = Command::new(executable);
    command.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec()).context(
            "platform command requires wafl-kube; run scripts/build.sh or use the WAFL image",
        )
    }
    #[cfg(not(unix))]
    {
        let status = command
            .status()
            .context("platform command requires wafl-kube")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
