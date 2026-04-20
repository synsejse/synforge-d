mod build;
mod git;

use std::sync::Arc;

use async_trait::async_trait;
use delegate::delegate;
use synforge_core::{api::PackageActionTargetResult, model::BuildTrigger};
use synforge_database::packages::PostgresPackageStore;
use synforge_git_sync::{RefreshAllProgressStore, RuntimeGitRegistryAdapter};
use synforge_state::RefreshAllPackagesProgressState;
use synforge_worker_host::{BuildQueue, BuildService, JobLifecycle, QueuedBuildRequest, WorkerBuildQueue};
use tracing::info;
use uuid::Uuid;

use crate::service::SynforgeService;

#[derive(Clone)]
pub(crate) struct DaemonPackageDeps {
    package_store: PostgresPackageStore,
    git: RuntimeGitRegistryAdapter,
    build_queue: WorkerBuildQueue,
    build_service: BuildService,
    lifecycle: Arc<JobLifecycle>,
    progress: RefreshAllPackagesProgressState,
}

impl DaemonPackageDeps {
    delegate! {
        to self.package_store {
            #[call(list_all_enabled_package_names)]
            async fn load_enabled_package_names(&self) -> anyhow::Result<Vec<String>>;

            #[call(list_published_repo_files_for_job)]
            async fn load_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<synforge_core::model::PublishedRepoFile>>;

            #[call(delete_job)]
            async fn remove_job_record(&self, job_id: Uuid) -> anyhow::Result<Option<synforge_core::api::BuildJobResponse>>;

            #[call(count_package_builds)]
            async fn load_package_build_count(&self, package_name: &str) -> anyhow::Result<u64>;

            #[call(list_package_builds)]
            async fn load_package_builds(&self, package_name: &str, limit: usize, offset: usize) -> anyhow::Result<Vec<synforge_core::api::BuildJobResponse>>;

            #[call(list_published_repo_files_for_package)]
            async fn load_published_repo_files_for_package(&self, package_name: &str) -> anyhow::Result<Vec<synforge_core::model::PublishedRepoFile>>;

            #[call(find_package)]
            async fn load_package(&self, package_name: &str) -> anyhow::Result<Option<synforge_core::api::PackageResponse>>;

            #[call(upsert_package_definition)]
            async fn save_package_definition(&self, package: &synforge_core::package::PackageDefinition) -> anyhow::Result<()>;

            #[call(has_active_job_for_target)]
            async fn load_has_active_job_for_target(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<bool>;

            #[call(get_last_successful_revision)]
            async fn load_last_successful_revision(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<Option<String>>;

            #[call(get_target_backoff_wait_seconds)]
            async fn load_target_backoff_wait_seconds(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<Option<u64>>;

            #[call(insert_build_job)]
            async fn save_build_job(&self, job: &synforge_core::model::BuildJob) -> anyhow::Result<()>;

            #[call(list_jobs_for_package)]
            async fn load_jobs_for_package(&self, package_name: &str) -> anyhow::Result<Vec<synforge_core::api::BuildJobResponse>>;
        }
    }

    delegate! {
        to self.git {
            #[call(browse_repository)]
            async fn browse_git_repository(&self, repo_url: &str) -> anyhow::Result<synforge_core::api::BrowseRepositoryResponse>;

            #[call(get_package)]
            async fn load_git_package(&self, package_name: &str) -> anyhow::Result<synforge_core::api::PackageResponse>;

            #[call(list_definitions)]
            async fn load_package_definitions(&self) -> anyhow::Result<Vec<synforge_core::package::PackageDefinition>>;

            #[call(get_definition)]
            async fn load_package_definition(&self, package_name: &str) -> anyhow::Result<synforge_core::package::PackageDefinition>;

            #[call(delete_package)]
            async fn delete_git_package(&self, package_name: &str) -> anyhow::Result<()>;
        }
    }

    delegate! {
        to self.build_queue {
            #[call(enqueue_build)]
            async fn queue_build_request(&self, build: QueuedBuildRequest) -> anyhow::Result<()>;
        }
    }
}

#[async_trait]
impl RefreshAllProgressStore for DaemonPackageDeps {
    async fn load_refresh_all_packages_progress(
        &self,
    ) -> Option<synforge_core::api::RefreshAllPackagesProgressView> {
        self.progress.load().await
    }

    async fn save_refresh_all_packages_progress(
        &self,
        progress: synforge_core::api::RefreshAllPackagesProgressView,
    ) {
        self.progress.save(progress).await;
    }
}

impl SynforgeService {
    pub(crate) fn package_deps(&self) -> DaemonPackageDeps {
        DaemonPackageDeps {
            package_store: PostgresPackageStore::new(self.store.clone()),
            git: self.registry.clone(),
            build_queue: WorkerBuildQueue::new(self.queue_tx.clone()),
            build_service: self.build_service.clone(),
            lifecycle: Arc::clone(&self.lifecycle),
            progress: self.refresh_all_packages_progress.clone(),
        }
    }
}

pub(super) fn log_action_response(
    package_name: &str,
    trigger: BuildTrigger,
    results: &[PackageActionTargetResult],
) {
    let mut queued = 0_usize;
    let mut skipped = 0_usize;
    let mut blocked = 0_usize;
    for result in results {
        match result.disposition {
            synforge_core::api::PackageActionDisposition::Queued => queued += 1,
            synforge_core::api::PackageActionDisposition::Skipped => skipped += 1,
            synforge_core::api::PackageActionDisposition::Blocked => blocked += 1,
        }
    }
    info!(
        package_name,
        trigger = ?trigger,
        queued_targets = queued,
        skipped_targets = skipped,
        blocked_targets = blocked,
        "manual package action scheduled"
    );
}
