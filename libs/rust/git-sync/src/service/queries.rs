//! Query-side operations for git synchronization.

use std::collections::HashMap;

use async_trait::async_trait;
use synforge_core::api::{
    BrowseRepositoryRequest, BrowseRepositoryResponse, BuildJobResponse,
    PackageBuildHistoryResponse, PackageBuildInventoryEntry, PackageResponse,
    PackageTargetCcacheStats, RefreshAllPackagesProgressResponse, RefreshAllPackagesProgressView,
};
use synforge_core::error::SynforgeError;
use synforge_core::model::PublishedRepoFile;
use uuid::Uuid;

use super::commands::RefreshAllProgressStore;

#[async_trait]
pub trait RepositoryBrowser {
    async fn browse_repository(&self, repo_url: &str) -> anyhow::Result<BrowseRepositoryResponse>;
}

#[async_trait]
pub trait PackageDetailsReader {
    async fn get_package(&self, package_name: &str) -> anyhow::Result<PackageResponse>;
}

#[async_trait]
pub trait PackageBuildHistoryReader {
    async fn count_package_builds(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<u64>;

    async fn list_package_builds(
        &self,
        package_name: &str,
        limit: usize,
        offset: usize,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn list_package_ccache_stats(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PackageTargetCcacheStats>>;
}

pub async fn get_package<D>(deps: &D, package_name: &str) -> anyhow::Result<PackageResponse>
where
    D: PackageDetailsReader + Send + Sync,
{
    deps.get_package(package_name).await
}

pub async fn browse_repository<D>(
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

pub async fn get_refresh_all_packages_progress<D>(
    deps: &D,
) -> anyhow::Result<RefreshAllPackagesProgressResponse>
where
    D: RefreshAllProgressStore + Send + Sync,
{
    let operation: Option<RefreshAllPackagesProgressView> =
        deps.load_refresh_all_packages_progress().await;
    Ok(RefreshAllPackagesProgressResponse { operation })
}

pub async fn get_package_build_history<D>(
    deps: &D,
    package_name: &str,
    limit: usize,
    offset: usize,
    include_deleted: bool,
) -> anyhow::Result<PackageBuildHistoryResponse>
where
    D: PackageBuildHistoryReader + PackageDetailsReader + Send + Sync,
{
    deps.get_package(package_name).await?;

    let (total, jobs, published_files, ccache_stats_by_target) = tokio::try_join!(
        deps.count_package_builds(package_name, include_deleted),
        deps.list_package_builds(package_name, limit, offset, include_deleted),
        deps.list_published_repo_files_for_package(package_name),
        deps.list_package_ccache_stats(package_name),
    )?;
    let returned = jobs.len();

    let mut published_files_by_job = HashMap::<Uuid, Vec<PublishedRepoFile>>::new();
    for file in &published_files {
        published_files_by_job
            .entry(file.job_id)
            .or_default()
            .push(file.clone());
    }

    let builds = jobs
        .into_iter()
        .map(|build| PackageBuildInventoryEntry {
            repo_files: published_files_by_job
                .remove(&build.job.id)
                .unwrap_or_default(),
            build,
        })
        .collect();

    Ok(PackageBuildHistoryResponse {
        package_name: package_name.to_string(),
        builds,
        ccache_stats_by_target,
        page: synforge_core::api::PageInfo {
            limit,
            offset,
            returned,
            total: Some(total),
            has_more: (offset as u64) + (returned as u64) < total,
        },
    })
}
