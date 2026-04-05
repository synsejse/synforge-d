use std::sync::Arc;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{BuildStatus, PublishedRepoFile, WorkerResult},
    package::PackageDefinition,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::{DieselStore, JobStore, RepoStore};
use crate::repo_manager::FileRepoManager;
use crate::scheduler::QueuedBuild;
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

    pub async fn mark_running(&self, job_id: Uuid, container_id: &str) -> anyhow::Result<()> {
        info!(
            job_id = %job_id,
            container_id = %container_id,
            "marking build as running"
        );
        self.store
            .set_job_running(job_id, Some(container_id))
            .await
            .with_context(|| format!("failed to persist running state for {}", job_id))
    }

    pub async fn fail_launch(
        &self,
        build: &QueuedBuild,
        error_message: &str,
    ) -> anyhow::Result<()> {
        error!(
            job_id = %build.job_id,
            package_name = %build.package.name,
            mock_chroot = %build.mock_chroot,
            error = error_message,
            "build launch failed"
        );
        self.store
            .finish_job(
                build.job_id,
                BuildStatus::Failed,
                Some(error_message),
                &[],
                &[],
            )
            .await
            .context("failed to persist failed build result")
    }

    pub async fn finalize_execution(
        &self,
        build: &QueuedBuild,
        execution: WorkerResult,
    ) -> anyhow::Result<()> {
        let WorkerResult::Build(build_result) = execution else {
            anyhow::bail!(
                "worker returned parse result for build job {}",
                build.job_id
            );
        };

        let mut status = build_result.status;
        let mut error_message = build_result.message.clone();
        let mut published_files = Vec::new();

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
                .publish_build(&build.package, &build_result, &self.config)
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

        self.store
            .finish_job(
                build.job_id,
                status,
                error_message.as_deref(),
                &build_result.artifacts,
                &published_files,
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
            .remove_build_files(files, &self.config)
            .await
    }

    pub async fn remove_job_runtime(&self, job_id: Uuid) -> anyhow::Result<()> {
        let workspace_dir = self.config.runtime_paths().job_root(job_id);
        if workspace_dir.exists() {
            tokio::fs::remove_dir_all(&workspace_dir)
                .await
                .with_context(|| format!("failed to remove {}", workspace_dir.display()))?;
        }
        Ok(())
    }

    pub async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()> {
        warn!(reason = message, "aborting unfinished jobs");
        self.store.abort_unfinished_jobs(message).await
    }

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
            let _ = self.store.delete_job(job_id).await?;
            self.remove_job_runtime(job_id).await?;
        }

        Ok(())
    }
}
