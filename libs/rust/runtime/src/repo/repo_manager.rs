use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, PublishedRepoFile, RepoPublication,
        WorkerBuildResult, format_timestamp, now_utc,
    },
    package::PackageDefinition,
};
use tokio::process::Command;
use tracing::{info, warn};
use uuid::Uuid;

use crate::repo::repo_signing::RepoSigningManager;
use crate::storage::JobObjectStorage;
use synforge_store::build_published_repo_path;

fn should_skip_artifact(artifact: &BuildArtifact, package: &PackageDefinition) -> bool {
    match artifact.kind {
        ArtifactKind::Srpm => !package.publish_srpm,
        ArtifactKind::Debuginfo | ArtifactKind::Debugsource => !package.publish_debuginfo,
        ArtifactKind::Rpm | ArtifactKind::Log | ArtifactKind::Other => false,
    }
}

#[derive(Debug, Clone)]
pub struct FileRepoManager {
    object_storage: JobObjectStorage,
}

impl FileRepoManager {
    pub fn new(object_storage: JobObjectStorage) -> Self {
        Self { object_storage }
    }

    pub async fn ensure_repo(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        info!(
            repo_dir = %config.runtime_paths().repo_dir().display(),
            "ensuring repository metadata exists"
        );
        self.object_storage
            .restore_repo_tree(config.runtime_paths().repo_dir())
            .await?;
        regenerate_metadata(config, &self.object_storage).await
    }

    pub async fn publish_build(
        &self,
        package: &PackageDefinition,
        worker_result: &WorkerBuildResult,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoPublication> {
        let paths = config.runtime_paths();
        if worker_result.status != BuildStatus::Succeeded {
            return Err(anyhow::anyhow!(
                "cannot publish artifacts for failed worker result"
            ));
        }
        info!(
            job_id = %worker_result.job_id,
            package_name = %package.name,
            artifact_count = worker_result.artifacts.len(),
            "publishing build output into repository"
        );
        let published_at = now_utc();
        let mut files = Vec::new();
        let mut seen_paths = HashSet::new();
        for artifact in &worker_result.artifacts {
            if should_skip_artifact(artifact, package) {
                continue;
            }
            let build_root = build_repo_build_dir(config, package, worker_result.job_id, artifact);
            tokio::fs::create_dir_all(&build_root)
                .await
                .with_context(|| format!("failed to create {}", build_root.display()))?;
            let source_path = paths
                .job_artifacts_dir(worker_result.job_id)
                .join(artifact.storage_path());
            let file_name = artifact.file.file_name().ok_or_else(|| {
                anyhow::anyhow!("artifact file {} has no filename", artifact.file.display())
            })?;
            let destination = build_root.join(file_name);
            let path = build_published_repo_path(
                &package.name,
                &artifact.mock_chroot,
                worker_result.job_id,
                &artifact.file,
            )?;
            if !seen_paths.insert(path.clone()) {
                continue;
            }
            if destination.exists() {
                fs::remove_file(&destination)
                    .with_context(|| format!("failed to replace {}", destination.display()))?;
            }
            fs::hard_link(&source_path, &destination).or_else(|_| {
                fs::copy(&source_path, &destination)
                    .map(|_| ())
                    .with_context(|| {
                        format!(
                            "failed to copy artifact {} to {}",
                            source_path.display(),
                            destination.display()
                        )
                    })
            })?;
            files.push(PublishedRepoFile {
                artifact_id: artifact.id,
                job_id: worker_result.job_id,
                package_name: package.name.clone(),
                mock_chroot: artifact.mock_chroot.clone(),
                path,
                sha256: artifact.sha256.clone(),
                size_bytes: artifact.size_bytes,
                kind: artifact.kind,
                published_at,
                signing_status: artifact.signing_status,
                signing_error_message: artifact.signing_error_message.clone(),
            });
        }
        regenerate_metadata(config, &self.object_storage).await?;
        info!(
            job_id = %worker_result.job_id,
            package_name = %package.name,
            published_file_count = files.len(),
            "repository publication completed"
        );
        Ok(RepoPublication {
            package_name: package.name.clone(),
            repo_root: paths.repo_dir().to_path_buf(),
            published_at,
            files,
        })
    }

    pub async fn remove_build_files(
        &self,
        files: &[PublishedRepoFile],
        config: &DaemonConfig,
    ) -> anyhow::Result<()> {
        let paths = config.runtime_paths();
        if !files.is_empty() {
            info!(
                file_count = files.len(),
                "removing repository files for pruned build history"
            );
        }
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
        regenerate_metadata(config, &self.object_storage).await?;
        if !files.is_empty() {
            info!(
                file_count = files.len(),
                "repository metadata regenerated after file removal"
            );
        }
        Ok(())
    }
}

fn build_repo_build_dir(
    config: &DaemonConfig,
    package: &PackageDefinition,
    job_id: Uuid,
    artifact: &BuildArtifact,
) -> std::path::PathBuf {
    config
        .runtime_paths()
        .repo_dir()
        .join("packages")
        .join(&package.name)
        .join(&artifact.mock_chroot)
        .join("builds")
        .join(job_id.to_string())
}

async fn regenerate_metadata(
    config: &DaemonConfig,
    object_storage: &JobObjectStorage,
) -> anyhow::Result<()> {
    let paths = config.runtime_paths();
    let repo_dir = paths.repo_dir();
    tokio::fs::create_dir_all(repo_dir).await?;
    info!(repo_dir = %repo_dir.display(), "regenerating repository metadata");
    let output = Command::new("createrepo_c")
        .arg("--update")
        .arg(repo_dir)
        .output()
        .await;
    let createrepo_result: anyhow::Result<()> = match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(anyhow::anyhow!(
            "createrepo_c failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let repodata = repo_dir.join("repodata");
            tokio::fs::create_dir_all(&repodata).await?;
            tokio::fs::write(
                repodata.join("repomd.xml"),
                format!("<repomd generated=\"{}\" />", format_timestamp(now_utc())),
            )
            .await?;
            warn!(
                repo_dir = %repo_dir.display(),
                "createrepo_c not found; wrote placeholder repomd.xml"
            );
            Ok(())
        }
        Err(error) => Err(error.into()),
    };
    createrepo_result?;
    let signing_manager = RepoSigningManager;
    signing_manager
        .reconcile_repo_metadata_signature(config, repo_dir)
        .await
        .with_context(|| {
            format!(
                "failed to reconcile repository metadata signing for {}",
                repo_dir.display()
            )
        })?;
    object_storage.sync_repo_tree(repo_dir).await?;
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
