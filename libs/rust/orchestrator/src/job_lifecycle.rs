use std::sync::Arc;

use anyhow::Context;
use synforge_core::{BuildStatus, DaemonConfig, PublishedRepoFile, WorkerResult};
use tracing::warn;
use uuid::Uuid;

use crate::db::{DieselStore, JobStore};
use crate::repo_manager::RepoManager;
use crate::scheduler::QueuedBuild;
use crate::workers::WorkerExecution;

#[derive(Clone)]
pub struct JobLifecycle {
    config: DaemonConfig,
    store: DieselStore,
    repo_manager: Arc<dyn RepoManager>,
}

impl JobLifecycle {
    pub fn new(config: DaemonConfig, store: DieselStore, repo_manager: Arc<dyn RepoManager>) -> Self {
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

    pub async fn fail_launch(&self, build: &QueuedBuild, error_message: &str) -> anyhow::Result<()> {
        self.store
            .finish_job(
                build.job_id,
                BuildStatus::Failed,
                Some(error_message),
                &[],
                &[],
                None,
            )
            .await
            .context("failed to persist failed build result")
    }

    pub async fn finalize_execution(
        &self,
        build: &QueuedBuild,
        execution: WorkerExecution,
    ) -> anyhow::Result<()> {
        let WorkerResult::Build(build_result) = execution.result else {
            anyhow::bail!("worker returned parse result for build job {}", build.job_id);
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
                build_result.logs_path.as_deref().or(execution.logs_path.as_deref()),
            )
            .await?;

        if status == BuildStatus::Succeeded {
            if let Err(error) = self.prune_old_build_history(&build.package, &build.mock_chroot).await {
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
        self.repo_manager.remove_build_files(files, &self.config).await
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
        package: &synforge_core::PackageDefinition,
        mock_chroot: &str,
    ) -> anyhow::Result<()> {
        let keep = package.package_history_count as usize;
        let jobs = self.store.list_jobs_for_package(&package.name).await?;
        let old_jobs = jobs
            .into_iter()
            .filter(|job| {
                job.job.status == BuildStatus::Succeeded && job.job.mock_chroot == mock_chroot
            })
            .skip(keep)
            .collect::<Vec<_>>();

        for job in old_jobs {
            let published_files = self.store.list_published_repo_files_for_job(job.job.id).await?;
            self.remove_published_files(&published_files).await?;
            let _ = self.store.delete_job(job.job.id).await?;
            self.remove_job_runtime(job.job.id).await?;
        }

        Ok(())
    }
}
