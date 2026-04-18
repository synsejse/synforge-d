mod build;
mod git;

use std::sync::Arc;

use async_trait::async_trait;
use synforge_core::{api::PackageActionTargetResult, model::BuildTrigger};
use synforge_database::packages::PostgresPackageStore;
use synforge_git_sync::{RefreshAllProgressStore, RuntimeGitRegistryAdapter};
use synforge_publish::JobObjectStorage;
use synforge_state::RefreshAllPackagesProgressState;
use synforge_worker_host::{BuildService, JobLifecycle, WorkerBuildQueue};
use tracing::info;

use crate::service::SynforgeService;

#[derive(Clone)]
pub(crate) struct DaemonPackageDeps {
    package_store: PostgresPackageStore,
    git: RuntimeGitRegistryAdapter,
    build_queue: WorkerBuildQueue,
    build_service: BuildService,
    lifecycle: Arc<JobLifecycle>,
    object_storage: JobObjectStorage,
    progress: RefreshAllPackagesProgressState,
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
            object_storage: self.object_storage.clone(),
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
