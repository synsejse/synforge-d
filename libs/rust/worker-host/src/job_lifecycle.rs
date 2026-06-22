use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Context;
use serde_json::Value;
use synforge_core::{
    config::DaemonConfig,
    model::{ArtifactSigningStatus, BuildStatus, PublishedRepoFile, WorkerResult},
    package::PackageDefinition,
};
use synforge_database::{DieselStore, JobStore, RepoStore};
use synforge_publish::{FileRepoManager, RepoSigningManager};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{QueuedBuild, WorkerJobTracker};

#[derive(Clone)]
pub struct JobLifecycle {
    config: DaemonConfig,
    store: DieselStore,
    repo_manager: Arc<FileRepoManager>,
}

impl JobLifecycle {
    pub fn new(
        config: DaemonConfig,
        store: DieselStore,
        repo_manager: Arc<FileRepoManager>,
    ) -> Self {
        Self {
            config,
            store,
            repo_manager,
        }
    }

    #[tracing::instrument(skip_all, fields(job_id = %job_id, container_id = %container_id))]
    pub async fn mark_running(&self, job_id: Uuid, container_id: &str) -> anyhow::Result<()> {
        info!("marking build as running");
        self.store
            .set_job_running(job_id, Some(container_id))
            .await
            .with_context(|| format!("failed to persist running state for {}", job_id))
    }

    pub async fn should_run_job(&self, job_id: Uuid) -> anyhow::Result<bool> {
        let Some(job) = self.store.get_job(job_id).await? else {
            return Ok(false);
        };
        Ok(job.job.status == BuildStatus::Pending)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            job_id = %build.job_id,
            package_name = %build.package.name,
            mock_chroot = %build.mock_chroot,
        )
    )]
    pub async fn fail_launch(
        &self,
        build: &QueuedBuild,
        error_message: &str,
    ) -> anyhow::Result<()> {
        error!(error = error_message, "build launch failed");
        let finalized = self
            .store
            .finish_job(
                build.job_id,
                BuildStatus::Failed,
                Some(error_message),
                &[],
                &[],
                &[],
            )
            .await
            .context("failed to persist failed build result")?;
        if !finalized {
            info!(
                job_id = %build.job_id,
                "job already finalized by another writer; skipping backoff update"
            );
            return Ok(());
        }
        self.store
            .update_build_failure_backoff(
                build.job_id,
                BuildStatus::Failed,
                self.config.build_failure_backoff_base_seconds,
                self.config.build_failure_backoff_max_seconds,
            )
            .await?;
        Ok(())
    }

    #[tracing::instrument(
        skip_all,
        fields(
            job_id = %build.job_id,
            package_name = %build.package.name,
            mock_chroot = %build.mock_chroot,
        )
    )]
    pub async fn finalize_execution(
        &self,
        build: &QueuedBuild,
        execution: WorkerResult,
    ) -> anyhow::Result<()> {
        let WorkerResult::Build(mut build_result) = execution else {
            anyhow::bail!(
                "worker returned parse result for build job {}",
                build.job_id
            );
        };

        let mut status = build_result.status;
        let mut error_message = build_result.message.clone();
        let mut published_files = Vec::new();
        let mut artifact_signatures = Vec::new();
        let effective_config = self.load_runtime_overrides().await?;

        if status == BuildStatus::Succeeded {
            let signing_manager = RepoSigningManager;
            artifact_signatures = signing_manager
                .sign_worker_artifacts_for_publication(
                    &effective_config,
                    &build.package,
                    &mut build_result,
                )
                .await;
            if artifact_signatures
                .iter()
                .any(|signature| signature.status == ArtifactSigningStatus::Failed)
            {
                status = BuildStatus::Failed;
                error_message =
                    Some("artifact signing failed for one or more published artifacts".to_string());
            }
        }

        if status == BuildStatus::Succeeded {
            info!(
                job_id = %build.job_id,
                package_name = %build.package.name,
                mock_chroot = %build.mock_chroot,
                artifact_count = build_result.artifacts.len(),
                "publishing build artifacts"
            );
            match self
                .repo_manager
                .publish_build(&build.package, &build_result, &effective_config)
                .await
            {
                Ok(publication) => {
                    info!(
                        job_id = %build.job_id,
                        package_name = %build.package.name,
                        mock_chroot = %build.mock_chroot,
                        published_file_count = publication.files.len(),
                        "build artifacts published"
                    );
                    published_files = publication.files;
                }
                Err(error) => {
                    error!(
                        job_id = %build.job_id,
                        package_name = %build.package.name,
                        mock_chroot = %build.mock_chroot,
                        error = %error,
                        "failed to publish build artifacts"
                    );
                    status = BuildStatus::Failed;
                    error_message = Some(error.to_string());
                }
            }
        }

        let finalized = self
            .store
            .finish_job(
                build.job_id,
                status,
                error_message.as_deref(),
                &build_result.artifacts,
                &published_files,
                &artifact_signatures,
            )
            .await?;
        if !finalized {
            // A racing finisher (e.g. a user-requested kill) already
            // transitioned this job out of an active state. Don't clobber
            // its recorded outcome or backoff bookkeeping.
            info!(
                job_id = %build.job_id,
                package_name = %build.package.name,
                mock_chroot = %build.mock_chroot,
                "job already finalized by another writer; skipping finalize"
            );
            return Ok(());
        }
        self.store
            .update_build_failure_backoff(
                build.job_id,
                status,
                self.config.build_failure_backoff_base_seconds,
                self.config.build_failure_backoff_max_seconds,
            )
            .await?;
        info!(
            job_id = %build.job_id,
            package_name = %build.package.name,
            mock_chroot = %build.mock_chroot,
            final_status = ?status,
            artifact_count = build_result.artifacts.len(),
            published_file_count = published_files.len(),
            "build finalized"
        );

        if status == BuildStatus::Succeeded
            && let Err(error) = self
                .prune_old_build_history(&build.package, &build.mock_chroot)
                .await
        {
            warn!(
                "failed to prune old build history for {} {}: {}",
                build.package.name, build.mock_chroot, error
            );
        }

        Ok(())
    }

    pub async fn remove_published_files(&self, files: &[PublishedRepoFile]) -> anyhow::Result<()> {
        if files.is_empty() {
            return Ok(());
        }
        info!(
            file_count = files.len(),
            "removing published repository files"
        );
        self.repo_manager
            .remove_build_files(files, &self.load_runtime_overrides().await?)
            .await
    }

    /// Best-effort removal of both per-job on-disk trees:
    ///   - `{runtime}/state/jobs/{job_id}/`  — daemon-visible artifacts/logs
    ///   - `{worker_jobs_root}/{job_id}/`    — worker-side bind-mounted copy
    ///
    /// Called when a job is soft-deleted or pruned. NotFound is fine
    /// (already gone). Other I/O errors are logged but not propagated:
    /// the DB soft-delete already succeeded, so leaking an orphan
    /// directory is preferable to wedging UI state with a half-applied
    /// delete.
    pub async fn remove_job_dir(&self, job_id: Uuid) {
        let runtime_dir = self.config.runtime_paths().job_root(job_id);
        remove_job_dir_best_effort(&runtime_dir, job_id).await;

        let worker_dir = self.config.worker_jobs_root().join(job_id.to_string());
        if worker_dir != runtime_dir {
            remove_job_dir_best_effort(&worker_dir, job_id).await;
        }
    }

    #[tracing::instrument(skip_all, fields(reason = %message))]
    pub async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()> {
        warn!("aborting unfinished jobs");
        self.store.abort_unfinished_jobs(message).await
    }

    #[tracing::instrument(skip_all, fields(package_name = %package.name, mock_chroot = %mock_chroot))]
    async fn prune_old_build_history(
        &self,
        package: &PackageDefinition,
        mock_chroot: &str,
    ) -> anyhow::Result<()> {
        let keep = package.package_history_count as usize;
        let old_job_ids = self
            .store
            .list_prunable_successful_job_ids(&package.name, mock_chroot, keep)
            .await?;
        if !old_job_ids.is_empty() {
            info!(
                package_name = %package.name,
                mock_chroot,
                prune_count = old_job_ids.len(),
                keep,
                "pruning old build history"
            );
        }

        for job_id in old_job_ids {
            let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
            self.remove_published_files(&published_files).await?;
            self.store.delete_job(job_id).await?;
            self.remove_job_dir(job_id).await;
        }

        Ok(())
    }

    async fn load_runtime_overrides(&self) -> anyhow::Result<DaemonConfig> {
        let mut config = self.config.clone();
        let settings = self.store.list_runtime_settings().await?;
        apply_bool_setting(
            &mut config.signing_enabled,
            settings.get("signing_enabled"),
            "signing_enabled",
        )?;
        apply_optional_string_setting(
            &mut config.signing_key_id,
            settings.get("signing_key_id"),
            "signing_key_id",
        )?;
        if config.signing_enabled {
            self.ensure_signing_key_from_runtime_settings(&mut config, &settings)
                .await?;
        }
        Ok(config)
    }

    async fn ensure_signing_key_from_runtime_settings(
        &self,
        config: &mut DaemonConfig,
        settings: &BTreeMap<String, Value>,
    ) -> anyhow::Result<()> {
        let Some(armored_private_key) = settings
            .get("signing_private_key_armored")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        let signing_manager = RepoSigningManager;
        let status = signing_manager.status(config).await?;
        if status.key_present {
            return Ok(());
        }
        signing_manager.remove_all_keys(config).await?;
        let imported = signing_manager
            .import_private_key(config, armored_private_key)
            .await?;
        if config.signing_key_id.as_deref() != Some(imported.key_id.as_str()) {
            config.signing_key_id = Some(imported.key_id.clone());
            let mut updates = BTreeMap::new();
            updates.insert("signing_key_id".to_string(), Value::String(imported.key_id));
            self.store.upsert_runtime_settings(updates).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl WorkerJobTracker for JobLifecycle {
    async fn mark_job_running(&self, job_id: Uuid, container_id: &str) -> anyhow::Result<()> {
        self.mark_running(job_id, container_id).await
    }
}

fn apply_bool_setting(target: &mut bool, value: Option<&Value>, key: &str) -> anyhow::Result<()> {
    if let Some(value) = value {
        let Some(value) = value.as_bool() else {
            anyhow::bail!("runtime setting must be a boolean: {key}");
        };
        *target = value;
    }
    Ok(())
}

async fn remove_job_dir_best_effort(dir: &std::path::Path, job_id: Uuid) {
    match tokio::fs::remove_dir_all(dir).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn!(
                job_id = %job_id,
                path = %dir.display(),
                error = %error,
                "failed to remove on-disk job directory after soft-delete"
            );
        }
    }
}

fn apply_optional_string_setting(
    target: &mut Option<String>,
    value: Option<&Value>,
    key: &str,
) -> anyhow::Result<()> {
    if let Some(value) = value {
        if value.is_null() {
            *target = None;
        } else {
            let Some(value) = value.as_str() else {
                anyhow::bail!("runtime setting must be a string or null: {key}");
            };
            let value = value.trim();
            if value.is_empty() {
                anyhow::bail!("runtime setting must not be empty when provided: {key}");
            }
            *target = Some(value.to_string());
        }
    }
    Ok(())
}
