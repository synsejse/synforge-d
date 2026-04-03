use std::sync::Arc;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{BuildStatus, PublishedRepoFile, WorkerResult},
    package::PackageDefinition,
};
use tracing::warn;
use uuid::Uuid;

use crate::db::{DieselStore, JobStore};
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
            match self
                .repo_manager
                .publish_build(&build.package, &build_result, &self.config)
                .await
            {
                Ok(publication) => published_files = publication.files,
                Err(error) => {
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

        if status == BuildStatus::Succeeded {
            if let Err(error) = self
                .prune_old_build_history(&build.package, &build.mock_chroot)
                .await
            {
                warn!(
                    "failed to prune old build history for {} {}: {}",
                    build.package.name, build.mock_chroot, error
                );
            }
        }

        Ok(())
    }

    pub async fn remove_published_files(&self, files: &[PublishedRepoFile]) -> anyhow::Result<()> {
        if files.is_empty() {
            return Ok(());
        }
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

        for job_id in old_job_ids {
            let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
            self.remove_published_files(&published_files).await?;
            let _ = self.store.delete_job(job_id).await?;
            self.remove_job_runtime(job_id).await?;
        }

        Ok(())
    }
}
