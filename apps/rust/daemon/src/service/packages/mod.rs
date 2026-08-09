mod actions;
mod deps;

use std::path::Path;

use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, CreatePackageRequest,
    MockChrootListResponse, PackageBuildHistoryResponse, PackageListResponse, PackageResponse,
    RefreshAllPackagesProgressResponse, RefreshAllPackagesResponse, UpdatePackageRequest,
    build_page_info, normalize_pagination,
};
use synforge_core::validated::PackageName;
use synforge_database::PackageStore;
use synforge_git_sync::{
    browse_repository as browse_git_repository, create_package as create_git_package,
    delete_package as delete_git_package, get_package as get_git_package,
    get_package_build_history as get_git_package_build_history,
    get_refresh_all_packages_progress as get_git_refresh_all_packages_progress,
    trigger_refresh_all_packages as trigger_git_refresh_all_packages,
    update_package as update_git_package,
};
use synforge_worker_host::MockChrootService;
use tracing::warn;

use super::SynforgeService;

impl SynforgeService {
    pub async fn list_packages(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<PackageListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self.store.count_packages(search.clone(), enabled).await?;
        let packages = self
            .store
            .list_packages(limit, offset, search, enabled)
            .await?;
        Ok(PackageListResponse {
            page: build_page_info(limit, offset, total, packages.len()),
            packages,
        })
    }

    pub async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse> {
        get_git_package(&self.package_deps(), package_name).await
    }

    pub async fn list_mock_chroots(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<MockChrootListResponse> {
        let full =
            MockChrootService::new(self.runtime_cache.clone(), self.mock_chroot_cache.clone())
                .list_mock_chroots(
                    &self.config.worker_image,
                    self.config.mock_chroot_cache_ttl_seconds,
                )
                .await?;
        let total = full.len();
        let (limit, offset) = normalize_pagination(limit, offset);
        let chroots: Vec<String> = full.into_iter().skip(offset).take(limit).collect();
        let page = build_page_info(limit, offset, total as u64, chroots.len());
        Ok(MockChrootListResponse { chroots, page })
    }

    pub async fn get_refresh_all_packages_progress(
        &self,
    ) -> anyhow::Result<RefreshAllPackagesProgressResponse> {
        get_git_refresh_all_packages_progress(&self.package_deps()).await
    }

    pub async fn trigger_refresh_all_packages(&self) -> anyhow::Result<RefreshAllPackagesResponse> {
        trigger_git_refresh_all_packages(&self.package_deps()).await
    }

    pub async fn get_package_build_history(
        &self,
        package_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
        include_deleted: bool,
    ) -> anyhow::Result<PackageBuildHistoryResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        get_git_package_build_history(
            &self.package_deps(),
            package_name,
            limit,
            offset,
            include_deleted,
        )
        .await
    }

    pub async fn create_package(
        &self,
        request: CreatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        create_git_package(&self.package_deps(), request).await
    }

    pub async fn browse_repository(
        &self,
        request: BrowseRepositoryRequest,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        browse_git_repository(&self.package_deps(), request).await
    }

    pub async fn update_package(
        &self,
        package_name: &str,
        request: UpdatePackageRequest,
    ) -> anyhow::Result<PackageResponse> {
        let response = update_git_package(&self.package_deps(), package_name, request).await?;
        // If the update leaves ccache disabled, drop the on-disk
        // cache tree for this package immediately. Without this the
        // dir would linger until the periodic orphan sweep runs.
        // (mock-cache stays — it's used regardless of the ccache flag.)
        if !response.package.ccache_enabled {
            remove_package_cache_subtree(&self.config.worker_ccache_root(), package_name, "ccache")
                .await;
        }
        Ok(response)
    }

    pub async fn delete_package(&self, package_name: &str) -> anyhow::Result<()> {
        PackageName::new(package_name)?;
        delete_git_package(&self.package_deps(), package_name).await?;
        // Now that the package row is gone, drop its on-disk
        // ccache and mock-cache trees. Best effort: a leaked dir is
        // strictly worse than the existing behaviour, but we won't
        // fail the delete if the FS removal trips on something
        // unexpected.
        self.cleanup_package_cache_dirs(package_name).await;
        Ok(())
    }

    pub(super) async fn cleanup_package_cache_dirs(&self, package_name: &str) {
        for (root, domain) in [
            (self.config.worker_ccache_root(), "ccache"),
            (self.config.worker_mock_cache_root(), "mock-cache"),
        ] {
            remove_package_cache_subtree(&root, package_name, domain).await;
        }
    }
}

async fn remove_package_cache_subtree(root: &Path, package_name: &str, domain: &'static str) {
    let dir = root.join(package_name);
    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn!(
                package_name,
                domain,
                path = %dir.display(),
                error = %error,
                "failed to remove package cache directory"
            );
        }
    }
}
