use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, PublishedRepoFile, RepoPublication,
        WorkerBuildResult, format_timestamp, now_utc,
    },
    package::{PackageDefinition, RepoTarget},
};
use synforge_database::build_published_repo_path;
use tokio::process::Command;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{JobObjectStorage, RepoSigningManager};

fn should_skip_artifact(artifact: &BuildArtifact, package: &PackageDefinition) -> bool {
    match artifact.kind {
        ArtifactKind::Srpm => !package.publish_srpm,
        ArtifactKind::Debuginfo | ArtifactKind::Debugsource => !package.publish_debuginfo,
        ArtifactKind::Rpm | ArtifactKind::Log | ArtifactKind::Other => false,
    }
}

fn should_restore_for_signing(artifact: &BuildArtifact, package: &PackageDefinition) -> bool {
    match artifact.kind {
        ArtifactKind::Rpm => true,
        ArtifactKind::Srpm => package.publish_srpm,
        ArtifactKind::Debuginfo | ArtifactKind::Debugsource => package.publish_debuginfo,
        ArtifactKind::Log | ArtifactKind::Other => false,
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
        let runtime_paths = config.runtime_paths();
        self.object_storage
            .restore_repo_tree(runtime_paths.repo_dir())
            .await?;
        for target in discover_repo_targets(runtime_paths.repo_dir()).await? {
            regenerate_target_metadata(runtime_paths.repo_target_dir(&target).as_path()).await?;
        }
        reconcile_repo_state(config, &self.object_storage).await
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
        let mut affected_targets = HashSet::new();
        for artifact in &worker_result.artifacts {
            if should_skip_artifact(artifact, package) {
                continue;
            }
            let build_root = build_repo_build_dir(config, package, worker_result.job_id, artifact)?;
            tokio::fs::create_dir_all(&build_root)
                .await
                .with_context(|| format!("failed to create {}", build_root.display()))?;
            let source_path = paths
                .job_artifacts_dir(worker_result.job_id)
                .join(artifact.storage_path());
            self.restore_artifact_if_missing(worker_result.job_id, artifact, &source_path)
                .await?;
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
            affected_targets.insert(target_repo_dir_from_mock_chroot(config, &artifact.mock_chroot)?);
            if tokio::fs::try_exists(&destination).await? {
                tokio::fs::remove_file(&destination)
                    .await
                    .with_context(|| format!("failed to replace {}", destination.display()))?;
            }
            if let Err(link_error) = tokio::fs::hard_link(&source_path, &destination).await {
                tokio::fs::copy(&source_path, &destination)
                    .await
                    .map(|_| ())
                    .with_context(|| {
                        format!(
                            "failed to link ({}) or copy artifact {} to {}",
                            link_error,
                            source_path.display(),
                            destination.display()
                        )
                    })?;
            }
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
        for target_repo_dir in affected_targets {
            regenerate_target_metadata(&target_repo_dir).await?;
        }
        reconcile_repo_state(config, &self.object_storage).await?;
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

    pub async fn ensure_worker_signing_artifacts_available(
        &self,
        package: &PackageDefinition,
        worker_result: &WorkerBuildResult,
        config: &DaemonConfig,
    ) -> anyhow::Result<()> {
        for artifact in &worker_result.artifacts {
            if !should_restore_for_signing(artifact, package) {
                continue;
            }
            let local_path = config
                .runtime_paths()
                .job_artifacts_dir(worker_result.job_id)
                .join(artifact.storage_path());
            self.restore_artifact_if_missing(worker_result.job_id, artifact, &local_path)
                .await?;
        }
        Ok(())
    }

    pub fn archive_worker_artifacts_in_background(
        &self,
        worker_result: &WorkerBuildResult,
        config: &DaemonConfig,
    ) {
        let object_storage = self.object_storage.clone();
        let job_id = worker_result.job_id;
        let uploads = worker_result
            .artifacts
            .iter()
            .map(|artifact| {
                let storage_path = artifact.storage_path().to_string_lossy().into_owned();
                let local_path = config
                    .runtime_paths()
                    .job_artifacts_dir(worker_result.job_id)
                    .join(artifact.storage_path());
                (storage_path, local_path)
            })
            .collect::<Vec<_>>();

        tokio::spawn(async move {
            for (storage_path, local_path) in uploads {
                if let Err(error) = object_storage
                    .store_job_artifact(job_id, &storage_path, &local_path)
                    .await
                {
                    warn!(
                        job_id = %job_id,
                        storage_path,
                        error = %error,
                        "failed to archive signed job artifact to object storage"
                    );
                }
            }
        });
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
        reconcile_repo_state(config, &self.object_storage).await?;
        if !files.is_empty() {
            info!(
                file_count = files.len(),
                "repository metadata regenerated after file removal"
            );
        }
        Ok(())
    }

    async fn restore_artifact_if_missing(
        &self,
        job_id: Uuid,
        artifact: &BuildArtifact,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        if tokio::fs::try_exists(local_path).await? {
            return Ok(());
        }
        let storage_path = artifact.storage_path().to_string_lossy().into_owned();
        let restored = self
            .object_storage
            .restore_job_artifact(job_id, &storage_path, local_path)
            .await?;
        if !restored {
            anyhow::bail!(
                "artifact {} is not available locally or in object storage",
                artifact.file.display()
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
) -> anyhow::Result<std::path::PathBuf> {
    Ok(target_repo_dir_from_mock_chroot(config, &artifact.mock_chroot)?
        .join("packages")
        .join(&package.name)
        .join("builds")
        .join(job_id.to_string()))
}

async fn regenerate_target_metadata(repo_dir: &Path) -> anyhow::Result<()> {
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
    createrepo_result
}

async fn reconcile_repo_state(
    config: &DaemonConfig,
    object_storage: &JobObjectStorage,
) -> anyhow::Result<()> {
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
    let object_storage = object_storage.clone();
    let repo_root = repo_root.to_path_buf();
    tokio::spawn(async move {
        if let Err(error) = object_storage.sync_repo_tree(&repo_root).await {
            warn!(
                repo_dir = %repo_root.display(),
                error = %error,
                "failed to sync repository tree to object storage"
            );
        }
    });
    Ok(())
}

pub(crate) async fn discover_repo_targets(repo_root: &Path) -> anyhow::Result<Vec<RepoTarget>> {
    let mut targets = Vec::new();
    let mut distro_dirs = match tokio::fs::read_dir(repo_root).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(targets),
        Err(error) => return Err(error.into()),
    };

    while let Some(distro_entry) = distro_dirs.next_entry().await? {
        let distro_path = distro_entry.path();
        if !distro_entry.file_type().await?.is_dir() {
            continue;
        }
        let mut release_dirs = tokio::fs::read_dir(&distro_path).await?;
        while let Some(release_entry) = release_dirs.next_entry().await? {
            if !release_entry.file_type().await?.is_dir() {
                continue;
            }
            let release_path = release_entry.path();
            let relative_path = release_path
                .strip_prefix(repo_root)
                .with_context(|| {
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

async fn refresh_target_repo_after_removal(repo_root: &Path, target_repo_dir: &Path) -> anyhow::Result<()> {
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
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        while let Some(entry) = entries.next_entry().await? {
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

fn target_repo_dir_from_mock_chroot(
    config: &DaemonConfig,
    mock_chroot: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let target = RepoTarget::from_mock_chroot(mock_chroot)
        .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", mock_chroot))
        ?;
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
