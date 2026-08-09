use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use anyhow::Context;
use strict_path::StrictPath;
use synforge_core::{
    config::DaemonConfig,
    model::{BuildStatus, PublishedRepoFile, RepoPublication, WorkerBuildResult, now_utc},
    package::PackageDefinition,
};
use synforge_database::build_published_repo_path;
use tracing::info;

use super::{
    FileRepoManager, build_repo_build_dir, ensure_createrepo_available, reconcile_repo_state,
    refresh_target_repo_after_removal, regenerate_target_metadata, should_skip_artifact,
    target_repo_dir_from_mock_chroot,
};
use crate::resolve_job_artifact_path;

struct PreparedArtifact {
    source: StrictPath,
    file_name: OsString,
    published: PublishedRepoFile,
}

impl FileRepoManager {
    pub async fn publish_build(
        &self,
        package: &PackageDefinition,
        worker_result: &WorkerBuildResult,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoPublication> {
        let _operation = self.operation_lock.lock().await;
        ensure_createrepo_available().await?;
        validate_worker_result(package, worker_result)?;

        info!(
            job_id = %worker_result.job_id,
            package_name = %package.name,
            artifact_count = worker_result.artifacts.len(),
            "staging build output for repository publication"
        );

        let published_at = now_utc();
        let mut prepared = Vec::new();
        let mut seen_paths = HashSet::new();
        let mut target_repo_dir = None;
        let mut final_build_dir = None;
        for artifact in &worker_result.artifacts {
            if artifact.package_name != package.name {
                anyhow::bail!(
                    "artifact {} belongs to package {}, expected {}",
                    artifact.id,
                    artifact.package_name,
                    package.name
                );
            }
            if should_skip_artifact(artifact, package) {
                continue;
            }

            let artifact_target = target_repo_dir_from_mock_chroot(config, &artifact.mock_chroot)?;
            ensure_single_path(&mut target_repo_dir, artifact_target, "repository target")?;
            let artifact_build_dir =
                build_repo_build_dir(config, package, worker_result.job_id, artifact)?;
            ensure_single_path(&mut final_build_dir, artifact_build_dir, "repository build")?;

            let source = resolve_job_artifact_path(config, worker_result.job_id, artifact)?;
            if !tokio::fs::try_exists(source.interop_path()).await? {
                anyhow::bail!(
                    "artifact {} is not available locally",
                    artifact.file.display()
                );
            }
            let file_name = artifact
                .file
                .file_name()
                .ok_or_else(|| {
                    anyhow::anyhow!("artifact file {} has no filename", artifact.file.display())
                })?
                .to_os_string();
            let path = build_published_repo_path(
                &package.name,
                &artifact.mock_chroot,
                worker_result.job_id,
                &artifact.file,
            )?;
            if !seen_paths.insert(path.clone()) {
                continue;
            }
            prepared.push(PreparedArtifact {
                source,
                file_name,
                published: PublishedRepoFile {
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
                },
            });
        }

        if prepared.is_empty() {
            return Ok(RepoPublication {
                package_name: package.name.clone(),
                repo_root: config.runtime_paths().repo_dir().to_path_buf(),
                published_at,
                files: Vec::new(),
            });
        }

        let target_repo_dir = target_repo_dir
            .ok_or_else(|| anyhow::anyhow!("prepared artifacts have no repository target"))?;
        let final_build_dir = final_build_dir
            .ok_or_else(|| anyhow::anyhow!("prepared artifacts have no build directory"))?;
        let staging = stage_artifacts(config, worker_result, &prepared).await?;
        let installed = install_staged_build(&final_build_dir, staging).await?;
        let publish_result = async {
            regenerate_target_metadata(&target_repo_dir).await?;
            reconcile_repo_state(config).await
        }
        .await;
        if let Err(error) = publish_result {
            let rollback = async {
                installed.rollback().await?;
                refresh_target_repo_after_removal(
                    config.runtime_paths().repo_dir(),
                    &target_repo_dir,
                )
                .await?;
                reconcile_repo_state(config).await
            }
            .await;
            return match rollback {
                Ok(()) => Err(error).context("repository publication rolled back"),
                Err(rollback_error) => Err(anyhow::anyhow!(
                    "repository publication failed ({error}); rollback also failed ({rollback_error})"
                )),
            };
        }
        installed.commit();

        let files = prepared
            .into_iter()
            .map(|artifact| artifact.published)
            .collect::<Vec<_>>();
        info!(
            job_id = %worker_result.job_id,
            package_name = %package.name,
            published_file_count = files.len(),
            "repository publication completed"
        );
        Ok(RepoPublication {
            package_name: package.name.clone(),
            repo_root: config.runtime_paths().repo_dir().to_path_buf(),
            published_at,
            files,
        })
    }
}

fn validate_worker_result(
    package: &PackageDefinition,
    worker_result: &WorkerBuildResult,
) -> anyhow::Result<()> {
    if worker_result.status != BuildStatus::Succeeded {
        anyhow::bail!("cannot publish artifacts for failed worker result");
    }
    if worker_result.package_name != package.name {
        anyhow::bail!(
            "worker result belongs to package {}, expected {}",
            worker_result.package_name,
            package.name
        );
    }
    Ok(())
}

fn ensure_single_path(
    selected: &mut Option<PathBuf>,
    candidate: PathBuf,
    label: &str,
) -> anyhow::Result<()> {
    match selected {
        Some(existing) if existing != &candidate => {
            anyhow::bail!(
                "build artifacts span more than one {label}: {} and {}",
                existing.display(),
                candidate.display()
            )
        }
        Some(_) => {}
        None => *selected = Some(candidate),
    }
    Ok(())
}

async fn stage_artifacts(
    config: &DaemonConfig,
    worker_result: &WorkerBuildResult,
    artifacts: &[PreparedArtifact],
) -> anyhow::Result<tempfile::TempDir> {
    let runtime_paths = config.runtime_paths();
    let repo_root = runtime_paths.repo_dir();
    let staging_parent = repo_root.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "repository root {} has no parent for staging",
            repo_root.display()
        )
    })?;
    let staging_root = staging_parent.join(".synforge-repo-publication");
    tokio::fs::create_dir_all(&staging_root).await?;
    let staging = tempfile::Builder::new()
        .prefix(&format!("{}-", worker_result.job_id))
        .tempdir_in(staging_root)?;
    let staged_build_dir = staging.path().join("build");
    tokio::fs::create_dir_all(&staged_build_dir).await?;

    for artifact in artifacts {
        let destination = staged_build_dir.join(&artifact.file_name);
        if let Err(link_error) =
            tokio::fs::hard_link(artifact.source.interop_path(), &destination).await
        {
            tokio::fs::copy(artifact.source.interop_path(), &destination)
                .await
                .map(|_| ())
                .with_context(|| {
                    format!(
                        "failed to link ({link_error}) or copy artifact {} to {}",
                        artifact.source.strictpath_display(),
                        destination.display()
                    )
                })?;
        }
    }
    Ok(staging)
}

struct InstalledBuild {
    _staging: tempfile::TempDir,
    final_build_dir: PathBuf,
    backup_build_dir: PathBuf,
    had_previous: bool,
}

impl InstalledBuild {
    async fn rollback(self) -> anyhow::Result<()> {
        match tokio::fs::remove_dir_all(&self.final_build_dir).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if self.had_previous {
            tokio::fs::rename(&self.backup_build_dir, &self.final_build_dir)
                .await
                .with_context(|| format!("failed to restore {}", self.final_build_dir.display()))?;
        }
        Ok(())
    }

    fn commit(self) {
        drop(self);
    }
}

async fn install_staged_build(
    final_build_dir: &std::path::Path,
    staging: tempfile::TempDir,
) -> anyhow::Result<InstalledBuild> {
    let staged_build_dir = staging.path().join("build");
    let backup_build_dir = staging.path().join("previous-build");
    let had_previous = tokio::fs::try_exists(final_build_dir).await?;
    if let Some(parent) = final_build_dir.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if had_previous {
        tokio::fs::rename(final_build_dir, &backup_build_dir)
            .await
            .with_context(|| format!("failed to back up {}", final_build_dir.display()))?;
    }
    if let Err(install_error) = tokio::fs::rename(&staged_build_dir, final_build_dir).await {
        let restore_result = if had_previous {
            tokio::fs::rename(&backup_build_dir, final_build_dir)
                .await
                .with_context(|| format!("failed to restore {}", final_build_dir.display()))
        } else {
            Ok(())
        };
        return match restore_result {
            Ok(()) => Err(install_error).with_context(|| {
                format!(
                    "failed to install staged repository build {}",
                    final_build_dir.display()
                )
            }),
            Err(restore_error) => Err(anyhow::anyhow!(
                "failed to install staged repository build {} ({install_error}); restoration also failed ({restore_error})",
                final_build_dir.display()
            )),
        };
    }

    Ok(InstalledBuild {
        _staging: staging,
        final_build_dir: final_build_dir.to_path_buf(),
        backup_build_dir,
        had_previous,
    })
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
