mod build;
mod git;

use std::sync::Arc;

use synforge_core::{
    api::{BrowseRepositoryResponse, BuildJobResponse, PackageResponse, PackageTargetCcacheStats},
    model::{BuildJob, PublishedRepoFile},
    package::PackageDefinition,
};
use synforge_database::{DieselStore, JobStore, PackageStore, RepoStore};
use synforge_git_sync::RuntimeGitRegistryAdapter;
use synforge_worker_host::{BuildQueue, JobLifecycle, QueuedBuildRequest, WorkerBuildQueue};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::service::SynforgeService;

#[derive(Clone)]
pub(crate) struct DaemonPackageDeps {
    store: DieselStore,
    git: RuntimeGitRegistryAdapter,
    build_queue: WorkerBuildQueue,
    lifecycle: Arc<JobLifecycle>,
}

impl DaemonPackageDeps {
    pub(super) async fn load_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store.list_published_repo_files_for_job(job_id).await
    }

    pub(super) async fn remove_job_record(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Option<BuildJobResponse>> {
        self.store.delete_job(job_id).await
    }

    pub(super) async fn load_package_build_count(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<u64> {
        self.store
            .count_jobs(
                None,
                Some(package_name.to_string()),
                None,
                false,
                include_deleted,
            )
            .await
    }

    pub(super) async fn load_package_builds(
        &self,
        package_name: &str,
        limit: usize,
        offset: usize,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_jobs(
                limit,
                offset,
                None,
                Some(package_name.to_string()),
                None,
                false,
                include_deleted,
            )
            .await
    }

    pub(super) async fn load_package_ccache_stats(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PackageTargetCcacheStats>> {
        self.store.list_package_ccache_stats(package_name).await
    }

    pub(super) async fn load_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store
            .list_published_repo_files_for_package(package_name)
            .await
    }

    pub(super) async fn load_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Option<PackageResponse>> {
        self.store.get_package(package_name).await
    }

    pub(super) async fn save_package_definition(
        &self,
        package: &PackageDefinition,
    ) -> anyhow::Result<()> {
        self.store.upsert_package(package).await
    }

    pub(super) async fn load_has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        self.store
            .has_active_job_for_target(package_name, mock_chroot)
            .await
    }

    pub(super) async fn load_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>> {
        self.store
            .get_last_successful_revision(package_name, mock_chroot)
            .await
    }

    /// Translate the raw backoff record into "seconds remaining until
    /// the target is eligible to build again", or `None` if no
    /// backoff is active. Powers UI countdowns and the sync schedule.
    pub(super) async fn load_target_backoff_wait_seconds(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<u64>> {
        let Some(backoff) = self
            .store
            .get_target_build_backoff(package_name, mock_chroot)
            .await?
        else {
            return Ok(None);
        };
        let now = OffsetDateTime::now_utc();
        if backoff.next_eligible_at <= now {
            return Ok(None);
        }
        Ok(Some(
            (backoff.next_eligible_at - now).whole_seconds().max(1) as u64,
        ))
    }

    pub(super) async fn save_build_job(&self, job: &BuildJob) -> anyhow::Result<bool> {
        self.store.insert_job(job).await
    }

    pub(super) async fn load_jobs_for_package(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_jobs_for_package(package_name, include_deleted)
            .await
    }

    pub(super) async fn browse_git_repository(
        &self,
        repo_url: &str,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        self.git.browse_repository(repo_url).await
    }

    pub(super) async fn load_git_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageResponse> {
        self.git.get_package(package_name).await
    }

    pub(super) async fn load_package_definition(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageDefinition> {
        self.git.get_definition(package_name).await
    }

    pub(super) async fn delete_git_package(&self, package_name: &str) -> anyhow::Result<()> {
        self.git.delete_package(package_name).await
    }

    pub(super) async fn queue_build_request(
        &self,
        build: QueuedBuildRequest,
    ) -> anyhow::Result<()> {
        self.build_queue.enqueue_build(build).await
    }
}

impl SynforgeService {
    pub(crate) fn package_deps(&self) -> DaemonPackageDeps {
        DaemonPackageDeps {
            store: self.store.clone(),
            git: self.registry.clone(),
            build_queue: WorkerBuildQueue::new(self.queue_tx.clone()),
            lifecycle: Arc::clone(&self.lifecycle),
        }
    }
}
