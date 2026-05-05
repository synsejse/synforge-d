use synforge_core::{
    api::{
        BrowseRepositoryRequest, BrowseRepositoryResponse, CreatePackageRequest,
        PackageBuildHistoryResponse, PackageResponse, RefreshAllPackagesProgressResponse,
        RefreshAllPackagesResponse, UpdatePackageRequest,
    },
    error::SynforgeError,
};

use super::{
    commands::{
        EnabledPackageCatalog, ManualRefreshScheduler, PackageDefinitionMaterializer,
        PackageDefinitionWriter, PackageDeleter, PackageDeletionJobReader, PackageDeletionRunner,
        PackageLookup, PackageSourceInspector, RefreshAllProgressStore, create_package,
        delete_package, trigger_refresh_all_packages, update_package,
    },
    queries::{
        PackageBuildHistoryReader, PackageDetailsReader, RepositoryBrowser,
        get_package_build_history, get_refresh_all_packages_progress,
    },
};

#[derive(Debug, Clone, Default)]
pub struct GitSyncService;

impl GitSyncService {
    pub async fn get_package<D>(
        &self,
        deps: &D,
        package_name: &str,
    ) -> anyhow::Result<PackageResponse>
    where
        D: PackageDetailsReader + Send + Sync,
    {
        deps.get_package(package_name).await
    }

    pub async fn get_refresh_all_packages_progress<D>(
        &self,
        deps: &D,
    ) -> anyhow::Result<RefreshAllPackagesProgressResponse>
    where
        D: RefreshAllProgressStore + Send + Sync,
    {
        get_refresh_all_packages_progress(deps).await
    }

    pub async fn trigger_refresh_all_packages<D>(
        &self,
        deps: &D,
    ) -> anyhow::Result<RefreshAllPackagesResponse>
    where
        D: EnabledPackageCatalog + ManualRefreshScheduler + RefreshAllProgressStore + Send + Sync,
    {
        trigger_refresh_all_packages(deps).await
    }

    pub async fn create_package<D>(
        &self,
        deps: &D,
        request: CreatePackageRequest,
    ) -> anyhow::Result<PackageResponse>
    where
        D: PackageLookup
            + PackageDefinitionWriter
            + PackageSourceInspector
            + PackageDefinitionMaterializer
            + PackageDetailsReader
            + Send
            + Sync,
    {
        create_package(deps, request).await
    }

    pub async fn get_package_build_history<D>(
        &self,
        deps: &D,
        package_name: &str,
        limit: usize,
        offset: usize,
        include_deleted: bool,
    ) -> anyhow::Result<PackageBuildHistoryResponse>
    where
        D: PackageBuildHistoryReader + PackageDetailsReader + Send + Sync,
    {
        get_package_build_history(deps, package_name, limit, offset, include_deleted).await
    }

    pub async fn browse_repository<D>(
        &self,
        deps: &D,
        request: BrowseRepositoryRequest,
    ) -> anyhow::Result<BrowseRepositoryResponse>
    where
        D: RepositoryBrowser + Send + Sync,
    {
        let repo_url = request.repo_url.trim();
        if repo_url.is_empty() {
            return Err(anyhow::anyhow!(SynforgeError::BadRequest(
                "repository URL is required".to_string()
            )));
        }

        deps.browse_repository(repo_url).await
    }

    pub async fn update_package<D>(
        &self,
        deps: &D,
        package_name: &str,
        request: UpdatePackageRequest,
    ) -> anyhow::Result<PackageResponse>
    where
        D: PackageDefinitionWriter
            + PackageSourceInspector
            + PackageDefinitionMaterializer
            + PackageDetailsReader
            + Send
            + Sync,
    {
        update_package(deps, package_name, request).await
    }

    pub async fn delete_package<D>(&self, deps: &D, package_name: &str) -> anyhow::Result<()>
    where
        D: PackageDeletionJobReader + PackageDeletionRunner + PackageDeleter + Send + Sync,
    {
        delete_package(deps, package_name).await
    }
}
