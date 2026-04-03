use anyhow::Context;
use synforge_core::{
    api::{BrowseRepositoryResponse, CreatePackageRequest, PackageResponse, UpdatePackageRequest},
    error::SynforgeError,
    package::{BuildEnvVar, PackageDefinition, SpecRevision, SpecSource},
};

use crate::db::{DieselStore, JobStore};
use crate::packages::{InspectedPackageSource, PackageSyncStore};

#[derive(Clone)]
pub struct PackageRegistry {
    store: DieselStore,
    package_store: PackageSyncStore,
}

impl PackageRegistry {
    pub fn new(store: DieselStore, package_store: PackageSyncStore) -> Self {
        Self { store, package_store }
    }

    pub async fn list_packages(&self, limit: usize, offset: usize) -> anyhow::Result<Vec<PackageResponse>> {
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
                true,
                request.network_access,
                request.mock_chroots,
                request.poll_interval_seconds,
                request.build_timeout_seconds,
                request.package_history_count,
                request.build_env,
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
        let build_env = request
            .build_env
            .unwrap_or_else(|| existing.package.build_env.clone());
        let mock_chroots = request
            .mock_chroots
            .unwrap_or_else(|| existing.package.mock_chroots.clone());
        let network_access = request
            .network_access
            .unwrap_or(existing.package.network_access);
        let (package, _) = self
            .package_store
            .sync_source(
                package_name,
                &request.source,
                enabled,
                network_access,
                mock_chroots,
                poll_interval_seconds,
                build_timeout_seconds,
                package_history_count,
                build_env,
            )
            .await?;
        if package.name != package_name {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "updated package name must match existing package".to_string(),
            )));
        }
        let mut package = package;
        package.publish_srpm = request
            .publish_srpm
            .unwrap_or(existing.package.publish_srpm);
        package.network_access = network_access;
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

    pub async fn materialize_inspected_source(
        &self,
        source: &SpecSource,
        inspected: &InspectedPackageSource,
        enabled: bool,
        network_access: bool,
        mock_chroots: Vec<String>,
        poll_interval_seconds: u64,
        build_timeout_seconds: u64,
        package_history_count: u64,
        build_env: Vec<BuildEnvVar>,
    ) -> anyhow::Result<PackageDefinition> {
        self.package_store
            .materialize(
                source,
                inspected,
                enabled,
                network_access,
                mock_chroots,
                poll_interval_seconds,
                build_timeout_seconds,
                package_history_count,
                build_env,
            )
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
                package.enabled,
                package.network_access,
                package.mock_chroots.clone(),
                package.poll_interval_seconds,
                package.build_timeout_seconds,
                package.package_history_count,
                package.build_env.clone(),
            )
            .await
    }

    pub async fn browse_repository(&self, repo_url: &str) -> anyhow::Result<BrowseRepositoryResponse> {
        self.package_store.browse_repository(repo_url).await
    }
}
