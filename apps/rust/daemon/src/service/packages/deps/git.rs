use async_trait::async_trait;
use synforge_core::{
    api::{BrowseRepositoryResponse, PackageActionResponse, PackageResponse},
    model::BuildTrigger,
    package::{PackageDefinition, SpecSource},
};
use synforge_git_sync::{
    InspectedPackageSource as GitSyncInspectedPackageSource,
    ManualRefreshScheduler as GitSyncManualRefreshScheduler,
    PackageDefinitionMaterializer as GitSyncPackageDefinitionMaterializer,
    PackageDeleter as GitSyncPackageDeleter, PackageDetailsReader as GitSyncPackageDetailsReader,
    PackageMaterializationOptions as GitSyncPackageMaterializationOptions,
    PackageSourceInspector as GitSyncPackageSourceInspector,
    RepositoryBrowser as GitSyncRepositoryBrowser,
};
use synforge_worker_host::{
    PackageDefinitionCatalog, PackageDefinitionReader, TrackedSourceInspector,
};

use super::DaemonPackageDeps;

#[async_trait]
impl GitSyncManualRefreshScheduler for DaemonPackageDeps {
    async fn enqueue_manual_refresh(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageActionResponse> {
        self.build_service
            .trigger_package_action(self, package_name, BuildTrigger::ManualRefresh, false)
            .await
    }
}

#[async_trait]
impl GitSyncRepositoryBrowser for DaemonPackageDeps {
    async fn browse_repository(&self, repo_url: &str) -> anyhow::Result<BrowseRepositoryResponse> {
        self.browse_git_repository(repo_url).await
    }
}

#[async_trait]
impl GitSyncPackageDetailsReader for DaemonPackageDeps {
    async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        self.load_git_package(package_name).await
    }
}

#[async_trait]
impl PackageDefinitionCatalog for DaemonPackageDeps {
    async fn list_package_definitions(&self) -> anyhow::Result<Vec<PackageDefinition>> {
        self.load_package_definitions().await
    }
}

#[async_trait]
impl GitSyncPackageSourceInspector for DaemonPackageDeps {
    async fn inspect_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
    ) -> anyhow::Result<GitSyncInspectedPackageSource> {
        let inspected = self
            .git
            .inspect_source(package_name, source, timeout_seconds)
            .await?;
        Ok(GitSyncInspectedPackageSource {
            package_name: inspected.package_name,
            description: inspected.description,
            revision: inspected.revision,
        })
    }
}

#[async_trait]
impl GitSyncPackageDefinitionMaterializer for DaemonPackageDeps {
    async fn materialize_package_definition(
        &self,
        source: &SpecSource,
        inspected: &GitSyncInspectedPackageSource,
        options: GitSyncPackageMaterializationOptions,
    ) -> anyhow::Result<PackageDefinition> {
        self.git
            .materialize_inspected_source(
                source,
                &synforge_git_sync::RuntimeInspectedPackageSource {
                    package_name: inspected.package_name.clone(),
                    description: inspected.description.clone(),
                    revision: inspected.revision.clone(),
                },
                synforge_git_sync::RuntimeMaterializePackageOptions {
                    enabled: options.enabled,
                    publish_srpm: options.publish_srpm,
                    publish_debuginfo: options.publish_debuginfo,
                    network_access: options.network_access,
                    ccache_enabled: options.ccache_enabled,
                    ccache_max_size_mb: options.ccache_max_size_mb,
                    mock_chroots: options.mock_chroots,
                    poll_interval_seconds: options.poll_interval_seconds,
                    build_timeout_seconds: options.build_timeout_seconds,
                    package_history_count: options.package_history_count,
                    cpu_limit_millicores: options.cpu_limit_millicores,
                    memory_limit_mb: options.memory_limit_mb,
                    build_env: options.build_env,
                },
            )
            .await
    }
}

#[async_trait]
impl PackageDefinitionReader for DaemonPackageDeps {
    async fn get_package_definition(
        &self,
        package_name: &str,
    ) -> anyhow::Result<PackageDefinition> {
        self.load_package_definition(package_name).await
    }
}

#[async_trait]
impl TrackedSourceInspector for DaemonPackageDeps {
    async fn inspect_source_tracked(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
        trigger: synforge_core::sync::SyncTriggerType,
    ) -> anyhow::Result<GitSyncInspectedPackageSource> {
        let inspected = self
            .git
            .inspect_source_tracked(package_name, source, timeout_seconds, trigger)
            .await?;
        Ok(GitSyncInspectedPackageSource {
            package_name: inspected.package_name,
            description: inspected.description,
            revision: inspected.revision,
        })
    }
}

#[async_trait]
impl GitSyncPackageDeleter for DaemonPackageDeps {
    async fn delete_package(&self, package_name: &str) -> anyhow::Result<()> {
        self.delete_git_package(package_name).await
    }
}
