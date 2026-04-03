use std::path::Path;

use anyhow::Context;
use synforge_core::package::SpecSource;
use tokio::process::Command;
use tracing::warn;

pub(crate) async fn clone_repository(
    source: &SpecSource,
    destination: &Path,
    commit: Option<&str>,
) -> anyhow::Result<()> {
    if tokio::fs::try_exists(destination).await? {
        if let Err(error) = tokio::fs::remove_dir_all(destination).await {
            warn!(
                "failed to remove existing checkout {}: {}",
                destination.display(),
                error
            );
        }
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    run_command(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(&source.repo_url)
            .arg(destination),
    )
    .await
    .with_context(|| format!("failed to clone {}", source.repo_url))?;

    if let Some(commit) = commit {
        run_command(
            Command::new("git")
                .current_dir(destination)
                .arg("fetch")
                .arg("--depth")
                .arg("1")
                .arg("origin")
                .arg(commit),
        )
        .await
        .with_context(|| format!("failed to fetch commit {}", commit))?;
        run_command(
            Command::new("git")
                .current_dir(destination)
                .arg("checkout")
                .arg("--detach")
                .arg(commit),
        )
        .await
        .with_context(|| format!("failed to checkout commit {}", commit))?;
    }
    Ok(())
}

pub(crate) async fn git_rev_parse(repo_dir: &Path, rev: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .arg("rev-parse")
        .arg(rev)
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse {} failed: {}",
            rev,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) async fn run_command(command: &mut Command) -> anyhow::Result<()> {
    let output = command.output().await?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}
