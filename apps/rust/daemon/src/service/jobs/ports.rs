use std::{path::Path, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use delegate::delegate;
use synforge_core::api::BuildJobResponse;
use synforge_core::config::DaemonConfig;
use synforge_core::error::SynforgeError;
use synforge_core::model::BuildTrigger;
use synforge_core::package::{PackageDefinition, SpecRevision};
use synforge_core::sync::SyncTriggerType;
use synforge_database::DieselStore;
use synforge_database::jobs::PostgresJobStore;
use synforge_database::packages::PostgresPackageStore;
use synforge_git_sync::RuntimeGitRegistryAdapter;
use synforge_worker_host::{
    ActiveTargetBuildReader, BuildJobReader, BuildQueue, DockerWorkerLauncher,
    ExistingSourceSyncer, JobLifecycle, PackageDefinitionReader, QueuedBuild, QueuedBuildRequest,
    RetryBuildCleaner, RetryJobResetter, WorkerBuildQueue,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::SynforgeService;

#[derive(Clone)]
pub(super) struct JobRetryDeps {
    config: DaemonConfig,
    store: DieselStore,
    registry: RuntimeGitRegistryAdapter,
    lifecycle: Arc<JobLifecycle>,
    worker_launcher: Arc<DockerWorkerLauncher>,
    queue_tx: mpsc::Sender<QueuedBuild>,
}

#[async_trait]
impl BuildJobReader for JobRetryDeps {
    async fn get_build_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        self.load_build_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }
}

#[async_trait]
impl PackageDefinitionReader for JobRetryDeps {
    async fn get_package_definition(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageDefinition> {
        self.load_package_definition(package_name).await
    }
}

#[async_trait]
impl ExistingSourceSyncer for JobRetryDeps {
    async fn sync_existing_source_tracked(
        &self,
        package: &PackageDefinition,
        trigger: SyncTriggerType,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)> {
        self.sync_tracked_source(package, trigger).await
    }
}

#[async_trait]
impl ActiveTargetBuildReader for JobRetryDeps {
    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        self.target_has_active_job(package_name, mock_chroot).await
    }
}

#[async_trait]
impl RetryBuildCleaner for JobRetryDeps {
    async fn cleanup_retry_build(&self, job_id: Uuid) -> anyhow::Result<()> {
        let published_files = self.load_published_repo_files_for_job(job_id).await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        self.worker_launcher.cleanup_session(job_id);
        self.cleanup_retry_runtime_dirs(job_id).await
    }
}

#[async_trait]
impl RetryJobResetter for JobRetryDeps {
    async fn reset_job_for_retry(
        &self,
        job_id: Uuid,
        trigger: BuildTrigger,
        revision: &str,
    ) -> anyhow::Result<()> {
        self.reset_retry_job(job_id, trigger, revision).await
    }
}

#[async_trait]
impl BuildQueue for JobRetryDeps {
    async fn enqueue_build(&self, build: QueuedBuildRequest) -> anyhow::Result<()> {
        self.queue_retry_build(build).await
    }
}

impl JobRetryDeps {
    fn job_store(&self) -> PostgresJobStore {
        PostgresJobStore::new(self.store.clone())
    }

    fn package_store(&self) -> PostgresPackageStore {
        PostgresPackageStore::new(self.store.clone())
    }

    fn build_queue(&self) -> WorkerBuildQueue {
        WorkerBuildQueue::new(self.queue_tx.clone())
    }

    delegate! {
        to self.job_store() {
            #[call(get_job)]
            async fn load_build_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;

            #[call(list_published_repo_files_for_job)]
            async fn load_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<synforge_core::model::PublishedRepoFile>>;

            #[call(reset_job_for_retry)]
            async fn reset_retry_job(&self, job_id: Uuid, trigger: BuildTrigger, revision: &str) -> anyhow::Result<()>;
        }
    }

    delegate! {
        to self.package_store() {
            #[call(has_active_job_for_target)]
            async fn target_has_active_job(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<bool>;
        }
    }

    delegate! {
        to self.registry {
            #[call(get_definition)]
            async fn load_package_definition(&self, package_name: &str) -> anyhow::Result<PackageDefinition>;

            #[call(sync_existing_source_tracked)]
            async fn sync_tracked_source(&self, package: &PackageDefinition, trigger: SyncTriggerType) -> anyhow::Result<(PackageDefinition, SpecRevision)>;
        }
    }

    delegate! {
        to self.build_queue() {
            #[call(enqueue_build)]
            async fn queue_retry_build(&self, build: QueuedBuildRequest) -> anyhow::Result<()>;
        }
    }

    async fn cleanup_retry_runtime_dirs(&self, job_id: Uuid) -> anyhow::Result<()> {
        let runtime_root = self.config.runtime_paths().job_root(job_id);
        remove_retry_runtime_dir(&runtime_root).await?;

        let worker_root = self.config.worker_jobs_root().join(job_id.to_string());
        if worker_root != runtime_root {
            remove_retry_runtime_dir(&worker_root).await?;
        }

        Ok(())
    }
}

impl SynforgeService {
    pub(super) fn job_retry_deps(&self) -> JobRetryDeps {
        JobRetryDeps {
            config: self.config.clone(),
            store: self.store.clone(),
            registry: self.registry.clone(),
            lifecycle: Arc::clone(&self.lifecycle),
            worker_launcher: Arc::clone(&self.worker_launcher),
            queue_tx: self.queue_tx.clone(),
        }
    }
}

pub(super) async fn remove_retry_runtime_dir(path: &Path) -> anyhow::Result<()> {
    if !tokio::fs::try_exists(path).await? {
        return Ok(());
    }
    tokio::fs::remove_dir_all(path)
        .await
        .with_context(|| format!("failed to clean retry runtime directory {}", path.display()))
}
