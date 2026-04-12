use anyhow::Context;
use synforge_core::{
    api::{
        BrowseRepositoryProgressView, BrowseRepositoryResponse, CreatePackageRequest,
        PackageResponse, UpdatePackageRequest,
    },
    error::SynforgeError,
    package::{PackageDefinition, SpecRevision, SpecSource},
    sync::SyncTriggerType,
};

use crate::db::{DieselStore, PackageStore};
use crate::git_cache::GitMirrorCacheStatsSnapshot;
use crate::packages::{InspectedPackageSource, MaterializePackageOptions, PackageSyncStore};
use crate::sync_tracker::{SyncResult, SyncStatusTracker};

#[derive(Clone)]
pub struct PackageRegistry {
    store: DieselStore,
    package_store: PackageSyncStore,
    sync_tracker: Option<SyncStatusTracker>,
}

impl PackageRegistry {
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

    pub async fn create_package(
        &self,
        request: CreatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        let (package, _) = self
            .package_store
            .sync_source(
                &request.name,
                &request.source,
                MaterializePackageOptions {
                    enabled: request.enabled,
                    publish_srpm: request.publish_srpm,
                    publish_debuginfo: request.publish_debuginfo,
                    network_access: request.network_access,
                    ccache_enabled: request.ccache_enabled,
                    ccache_max_size_mb: request.ccache_max_size_mb.filter(|value| *value > 0),
                    mock_chroots: request.mock_chroots,
                    poll_interval_seconds: request.poll_interval_seconds,
                    build_timeout_seconds: request.build_timeout_seconds,
                    package_history_count: request.package_history_count,
                    cpu_limit_millicores: request.cpu_limit_millicores.filter(|value| *value > 0),
                    memory_limit_mb: request.memory_limit_mb.filter(|value| *value > 0),
                    build_env: request.build_env,
                },
            )
            .await?;
        if self.store.get_package(&request.name).await?.is_some() {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "package {} already exists",
                request.name
            ))));
        }
        self.store.upsert_package(&package).await?;
        self.get_package(&request.name).await
    }

    pub async fn update_package(
        &self,
        package_name: &str,
        request: UpdatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        let existing = self.get_package(package_name).await?;
        let enabled = request.enabled.unwrap_or(existing.package.enabled);
        let poll_interval_seconds = request
            .poll_interval_seconds
            .unwrap_or(existing.package.poll_interval_seconds);
        let build_timeout_seconds = request
            .build_timeout_seconds
            .unwrap_or(existing.package.build_timeout_seconds);
        let package_history_count = request
            .package_history_count
            .unwrap_or(existing.package.package_history_count);
        let cpu_limit_millicores = match request.cpu_limit_millicores {
            Some(0) => None,
            Some(value) => Some(value),
            None => existing.package.cpu_limit_millicores,
        };
        let memory_limit_mb = match request.memory_limit_mb {
            Some(0) => None,
            Some(value) => Some(value),
            None => existing.package.memory_limit_mb,
        };
        let build_env = request
            .build_env
            .unwrap_or_else(|| existing.package.build_env.clone());
        let mock_chroots = request
            .mock_chroots
            .unwrap_or_else(|| existing.package.mock_chroots.clone());
        let network_access = request
            .network_access
            .unwrap_or(existing.package.network_access);
        let ccache_enabled = request
            .ccache_enabled
            .unwrap_or(existing.package.ccache_enabled);
        let ccache_max_size_mb = match request.ccache_max_size_mb {
            Some(0) => None,
            Some(value) => Some(value),
            None => existing.package.ccache_max_size_mb,
        };
        let publish_srpm = request
            .publish_srpm
            .unwrap_or(existing.package.publish_srpm);
        let publish_debuginfo = request
            .publish_debuginfo
            .unwrap_or(existing.package.publish_debuginfo);
        let (package, _) = self
            .package_store
            .sync_source(
                package_name,
                &request.source,
                MaterializePackageOptions {
                    enabled,
                    publish_srpm,
                    publish_debuginfo,
                    network_access,
                    ccache_enabled,
                    ccache_max_size_mb,
                    mock_chroots,
                    poll_interval_seconds,
                    build_timeout_seconds,
                    package_history_count,
                    cpu_limit_millicores,
                    memory_limit_mb,
                    build_env,
                },
            )
            .await?;
        if package.name != package_name {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "updated package name must match existing package".to_string(),
            )));
        }
        self.store.upsert_package(&package).await?;
        self.get_package(package_name).await
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
    ) -> anyhow::Result<InspectedPackageSource> {
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
    ) -> anyhow::Result<InspectedPackageSource> {
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
        inspected: &InspectedPackageSource,
        options: MaterializePackageOptions,
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
                MaterializePackageOptions {
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

    pub async fn browse_repository_progress(&self) -> Option<BrowseRepositoryProgressView> {
        self.package_store.browse_repository_progress().await
    }

    pub async fn git_cache_stats(&self) -> anyhow::Result<GitMirrorCacheStatsSnapshot> {
        self.package_store.git_cache_stats().await
    }
}
