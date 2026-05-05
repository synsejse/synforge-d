mod cache;
mod package_store;
mod service;
mod sync_tracker;

use anyhow::Context;
use synforge_core::{
    api::{BrowseRepositoryResponse, PackageResponse},
    error::SynforgeError,
    package::{PackageDefinition, SpecRevision, SpecSource},
    sync::SyncTriggerType,
};
use synforge_database::{DieselStore, PackageStore};

pub use self::cache::GitMirrorCacheStatsSnapshot;
pub use self::package_store::PackageSyncStore;
use self::package_store::{
    InspectedPackageSource as RuntimeInspectedPackageSourceInner,
    MaterializePackageOptions as RuntimeMaterializePackageOptionsInner,
};
pub use self::package_store::{
    InspectedPackageSource as RuntimeInspectedPackageSource,
    MaterializePackageOptions as RuntimeMaterializePackageOptions, WorkerParseRunner,
};
pub use self::service::{
    EnabledPackageCatalog, GitSyncService, InspectedPackageSource, ManualRefreshScheduler,
    PackageBuildHistoryReader, PackageDefinitionMaterializer, PackageDefinitionWriter,
    PackageDeleter, PackageDeletionJobReader, PackageDeletionRunner, PackageDetailsReader,
    PackageLookup, PackageMaterializationOptions, PackageSourceInspector, RefreshAllProgressStore,
    RepositoryBrowser,
};
use self::sync_tracker::SyncResult;
pub use self::sync_tracker::SyncStatusTracker;

#[derive(Clone)]
pub struct RuntimeGitRegistryAdapter {
    store: DieselStore,
    package_store: PackageSyncStore,
    sync_tracker: Option<SyncStatusTracker>,
}

impl RuntimeGitRegistryAdapter {
    pub fn new(
        store: DieselStore,
        package_store: PackageSyncStore,
        sync_tracker: Option<SyncStatusTracker>,
    ) -> Self {
        Self {
            store,
            package_store,
            sync_tracker,
        }
    }

    pub async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<PackageResponse>> {
        self.store.list_packages(limit, offset, None, None).await
    }

    pub async fn list_definitions(&self) -> anyhow::Result<Vec<PackageDefinition>> {
        Ok(self
            .store
            .list_packages(10_000, 0, None, None)
            .await?
            .into_iter()
            .map(|response| response.package)
            .collect())
    }

    pub async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        self.store
            .get_package(package_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(package_name.to_string())))
    }

    pub async fn get_definition(&self, package_name: &str) -> anyhow::Result<PackageDefinition> {
        Ok(self.get_package(package_name).await?.package)
    }

    pub async fn delete_package(&self, package_name: &str) -> anyhow::Result<()> {
        self.package_store.delete(package_name).await?;
        self.store.remove_package(package_name).await?;
        Ok(())
    }

    pub async fn inspect_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
    ) -> anyhow::Result<RuntimeInspectedPackageSourceInner> {
        self.package_store
            .inspect_source(package_name, source, timeout_seconds)
            .await
    }

    pub async fn inspect_source_tracked(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
        trigger: SyncTriggerType,
    ) -> anyhow::Result<RuntimeInspectedPackageSourceInner> {
        let result = self
            .inspect_source(package_name, source, timeout_seconds)
            .await;

        if let Some(sync_tracker) = self.sync_tracker.clone() {
            let sync_result = match &result {
                Ok(inspected) => SyncResult::Success {
                    revision: inspected.revision.comparison_key(),
                },
                Err(error) => SyncResult::Failure {
                    error: error.to_string(),
                },
            };
            sync_tracker
                .record_sync_safe(package_name, trigger, &sync_result)
                .await;
        }

        result
    }

    pub async fn materialize_inspected_source(
        &self,
        source: &SpecSource,
        inspected: &RuntimeInspectedPackageSourceInner,
        options: RuntimeMaterializePackageOptionsInner,
    ) -> anyhow::Result<PackageDefinition> {
        self.package_store
            .materialize(source, inspected, options)
            .await
            .with_context(|| format!("failed to materialize package {}", inspected.package_name))
    }

    pub async fn sync_existing_source(
        &self,
        package: &PackageDefinition,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)> {
        self.package_store
            .sync_source(
                &package.name,
                &package.source,
                RuntimeMaterializePackageOptionsInner {
                    enabled: package.enabled,
                    publish_srpm: package.publish_srpm,
                    publish_debuginfo: package.publish_debuginfo,
                    network_access: package.network_access,
                    ccache_enabled: package.ccache_enabled,
                    ccache_max_size_mb: package.ccache_max_size_mb,
                    mock_chroots: package.mock_chroots.clone(),
                    poll_interval_seconds: package.poll_interval_seconds,
                    build_timeout_seconds: package.build_timeout_seconds,
                    package_history_count: package.package_history_count,
                    cpu_limit_millicores: package.cpu_limit_millicores,
                    memory_limit_mb: package.memory_limit_mb,
                    build_env: package.build_env.clone(),
                },
            )
            .await
    }

    pub async fn sync_existing_source_tracked(
        &self,
        package: &PackageDefinition,
        trigger: SyncTriggerType,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)> {
        let result = self.sync_existing_source(package).await;

        if let Some(sync_tracker) = self.sync_tracker.clone() {
            let sync_result = match &result {
                Ok((_, revision)) => SyncResult::Success {
                    revision: revision.comparison_key(),
                },
                Err(error) => SyncResult::Failure {
                    error: error.to_string(),
                },
            };
            sync_tracker
                .record_sync_safe(&package.name, trigger, &sync_result)
                .await;
        }

        result
    }

    pub async fn browse_repository(
        &self,
        repo_url: &str,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        self.package_store.browse_repository(repo_url).await
    }

    pub async fn git_cache_stats(&self) -> anyhow::Result<GitMirrorCacheStatsSnapshot> {
        self.package_store.git_cache_stats().await
    }
}
