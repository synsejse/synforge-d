use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use tokio::process::Command;

pub(super) async fn run_git(
    dir: Option<&Path>,
    args: &[&str],
    timeout: Duration,
) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(super) async fn run_git_clone(
    dir: Option<&Path>,
    args: &[&str],
    timeout: Duration,
) -> anyhow::Result<()> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
