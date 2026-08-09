use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{ArtifactKind, BuildArtifact, PublishedRepoFile},
    package::{PackageDefinition, RepoTarget},
};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_stream::{StreamExt, wrappers::ReadDirStream};
use tracing::info;
use uuid::Uuid;

use crate::RepoSigningManager;

#[path = "repo_manager/publication.rs"]
mod publication;

fn should_skip_artifact(artifact: &BuildArtifact, package: &PackageDefinition) -> bool {
    match artifact.kind {
        ArtifactKind::Srpm => !package.publish_srpm,
        ArtifactKind::Debuginfo | ArtifactKind::Debugsource => !package.publish_debuginfo,
        ArtifactKind::Rpm | ArtifactKind::Log | ArtifactKind::Other => false,
    }
}

#[derive(Debug, Clone)]
pub struct FileRepoManager {
    operation_lock: Arc<Mutex<()>>,
}

impl Default for FileRepoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileRepoManager {
    pub fn new() -> Self {
        Self {
            operation_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn ensure_repo(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        let _operation = self.operation_lock.lock().await;
        ensure_createrepo_available().await?;
        info!(
            repo_dir = %config.runtime_paths().repo_dir().display(),
            "ensuring repository metadata exists"
        );
        let runtime_paths = config.runtime_paths();
        for target in discover_repo_targets(runtime_paths.repo_dir()).await? {
            regenerate_target_metadata(runtime_paths.repo_target_dir(&target).as_path()).await?;
        }
        reconcile_repo_state(config).await
    }

    pub async fn remove_build_files(
        &self,
        files: &[PublishedRepoFile],
        config: &DaemonConfig,
    ) -> anyhow::Result<()> {
        let _operation = self.operation_lock.lock().await;
        let paths = config.runtime_paths();
        if !files.is_empty() {
            info!(
                file_count = files.len(),
                "removing repository files for pruned build history"
            );
        }
        let affected_targets = files
            .iter()
            .map(|file| target_repo_dir_from_relative_path(paths.repo_dir(), &file.path))
            .collect::<anyhow::Result<HashSet<_>>>()?;
        for file in files {
            let path = paths.repo_dir().join(&file.path);
            match tokio::fs::remove_file(&path).await {
                Ok(()) => prune_empty_parents(&path, paths.repo_dir()).await?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(anyhow::Error::from(error))
                        .context(format!("failed to remove {}", path.display()));
                }
            }
        }
        for target_repo_dir in affected_targets {
            refresh_target_repo_after_removal(paths.repo_dir(), &target_repo_dir).await?;
        }
        reconcile_repo_state(config).await?;
        if !files.is_empty() {
            info!(
                file_count = files.len(),
                "repository metadata regenerated after file removal"
            );
        }
        Ok(())
    }
}

pub(super) fn build_repo_build_dir(
    config: &DaemonConfig,
    package: &PackageDefinition,
    job_id: Uuid,
    artifact: &BuildArtifact,
) -> anyhow::Result<std::path::PathBuf> {
    Ok(
        target_repo_dir_from_mock_chroot(config, &artifact.mock_chroot)?
            .join("packages")
            .join(&package.name)
            .join("builds")
            .join(job_id.to_string()),
    )
}

pub(super) async fn ensure_createrepo_available() -> anyhow::Result<()> {
    let output = Command::new("createrepo_c")
        .arg("--version")
        .output()
        .await
        .context("createrepo_c is required for repository publication")?;
    if !output.status.success() {
        anyhow::bail!(
            "createrepo_c --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub(super) async fn regenerate_target_metadata(repo_dir: &Path) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(repo_dir).await?;
    info!(repo_dir = %repo_dir.display(), "regenerating repository metadata");
    let output = Command::new("createrepo_c")
        .arg("--update")
        .arg(repo_dir)
        .output()
        .await
        .context("createrepo_c is required for repository publication")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "createrepo_c failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(super) async fn reconcile_repo_state(config: &DaemonConfig) -> anyhow::Result<()> {
    let runtime_paths = config.runtime_paths();
    let repo_root = runtime_paths.repo_dir();
    clear_root_repo_metadata(repo_root).await?;
    let signing_manager = RepoSigningManager;
    signing_manager
        .reconcile_repo_metadata_signature(config, repo_root)
        .await
        .with_context(|| {
            format!(
                "failed to reconcile repository metadata signing for {}",
                repo_root.display()
            )
        })?;
    Ok(())
}

pub(crate) async fn discover_repo_targets(repo_root: &Path) -> anyhow::Result<Vec<RepoTarget>> {
    let mut targets = Vec::new();
    let mut distro_dirs = match tokio::fs::read_dir(repo_root).await {
        Ok(entries) => ReadDirStream::new(entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(targets),
        Err(error) => return Err(error.into()),
    };

    while let Some(distro_entry) = distro_dirs.next().await.transpose()? {
        let distro_path = distro_entry.path();
        if !distro_entry.file_type().await?.is_dir() {
            continue;
        }
        let mut release_dirs = ReadDirStream::new(tokio::fs::read_dir(&distro_path).await?);
        while let Some(release_entry) = release_dirs.next().await.transpose()? {
            if !release_entry.file_type().await?.is_dir() {
                continue;
            }
            let release_path = release_entry.path();
            let relative_path = release_path.strip_prefix(repo_root).with_context(|| {
                format!(
                    "failed to resolve repository target path {} under {}",
                    release_path.display(),
                    repo_root.display()
                )
            })?;
            let target = RepoTarget::from_repo_relative_path(relative_path).ok_or_else(|| {
                anyhow::anyhow!(
                    "repository target path {} does not match target layout",
                    relative_path.display()
                )
            })?;
            targets.push(target);
        }
    }

    Ok(targets)
}

pub(super) async fn refresh_target_repo_after_removal(
    repo_root: &Path,
    target_repo_dir: &Path,
) -> anyhow::Result<()> {
    if target_repo_has_published_files(target_repo_dir).await? {
        regenerate_target_metadata(target_repo_dir).await
    } else {
        match tokio::fs::remove_dir_all(target_repo_dir).await {
            Ok(()) => prune_empty_parents(target_repo_dir, repo_root).await,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

async fn target_repo_has_published_files(target_repo_dir: &Path) -> anyhow::Result<bool> {
    let packages_dir = target_repo_dir.join("packages");
    dir_contains_files(&packages_dir).await
}

async fn dir_contains_files(dir: &Path) -> anyhow::Result<bool> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(&current).await {
            Ok(entries) => ReadDirStream::new(entries),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        while let Some(entry) = entries.next().await.transpose()? {
            let file_type = entry.file_type().await?;
            if file_type.is_file() {
                return Ok(true);
            }
            if file_type.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    Ok(false)
}

pub(super) fn target_repo_dir_from_mock_chroot(
    config: &DaemonConfig,
    mock_chroot: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let target = RepoTarget::from_mock_chroot(mock_chroot)
        .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", mock_chroot))?;
    Ok(config.runtime_paths().repo_target_dir(&target))
}

fn target_repo_dir_from_relative_path(
    repo_root: &Path,
    relative_path: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let target = RepoTarget::from_repo_relative_path(relative_path).ok_or_else(|| {
        anyhow::anyhow!(
            "published repository path {} does not include a target root",
            relative_path.display()
        )
    })?;
    Ok(repo_root.join(target.repo_subdir()))
}

async fn clear_root_repo_metadata(repo_root: &Path) -> anyhow::Result<()> {
    let repodata_dir = repo_root.join("repodata");
    match tokio::fs::remove_dir_all(&repodata_dir).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

async fn prune_empty_parents(path: &Path, repo_root: &Path) -> anyhow::Result<()> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == repo_root {
            break;
        }
        match tokio::fs::remove_dir(dir).await {
            Ok(()) => current = dir.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => current = dir.parent(),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
