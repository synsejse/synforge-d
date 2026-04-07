use std::path::Path;

use anyhow::Context;
use synforge_core::package::SpecSource;
use tokio::process::Command;
use tracing::{info, warn};

const SHALLOW_CLONE_DEPTH: &str = "1";
const HEAD_REVISION: &str = "HEAD";

pub(crate) async fn clone_repository(
    source: &SpecSource,
    destination: &Path,
    commit: Option<&str>,
    git_mirror_reference: Option<&str>,
) -> anyhow::Result<()> {
    info!(
        repo_url = %source.repo_url,
        destination = %destination.display(),
        commit = ?commit,
        "cloning package source repository"
    );
    if tokio::fs::try_exists(destination).await?
        && let Err(error) = tokio::fs::remove_dir_all(destination).await
    {
        warn!(
            "failed to remove existing checkout {}: {}",
            destination.display(),
            error
        );
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut clone_command = Command::new("git");
    clone_command
        .arg("clone")
        .arg("--depth")
        .arg(SHALLOW_CLONE_DEPTH);
    if let Some(mirror_reference) = git_mirror_reference
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && tokio::fs::try_exists(mirror_reference)
            .await
            .unwrap_or(false)
    {
        clone_command
            .arg("--reference-if-able")
            .arg(mirror_reference);
    }
    clone_command.arg(&source.repo_url).arg(destination);
    run_command(&mut clone_command)
        .await
        .with_context(|| format!("failed to clone {}", source.repo_url))?;

    if let Some(commit) = commit {
        run_command(
            Command::new("git")
                .current_dir(destination)
                .arg("fetch")
                .arg("--depth")
                .arg(SHALLOW_CLONE_DEPTH)
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
    info!(
        repo_url = %source.repo_url,
        destination = %destination.display(),
        commit = ?commit,
        "repository clone prepared"
    );
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

pub(crate) async fn git_head_commit(repo_dir: &Path) -> anyhow::Result<String> {
    git_rev_parse(repo_dir, HEAD_REVISION).await
}

pub(crate) async fn run_command(command: &mut Command) -> anyhow::Result<()> {
    let output = command.output().await?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}
