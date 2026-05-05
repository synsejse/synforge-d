use std::path::{Path, PathBuf};

use async_trait::async_trait;
use synforge_core::{
    api::{
        PageInfo, RepoInventoryResponse, RepoSigningReconcileProgressResponse,
        RepoSigningReconcileProgressView, RepoSigningStatusResponse, RepoSigningStatusView,
        RepoSummaryResponse,
    },
    config::DaemonConfig,
    error::SynforgeError,
    model::{ArtifactKind, PublishedRepoFile},
};

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;

#[async_trait]
pub trait RepoInventoryReader {
    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64>;

    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;
}

#[async_trait]
pub trait RepoSummaryReader {
    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)>;

    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64>;

    async fn count_all_published_repo_files(&self) -> anyhow::Result<u64>;

    async fn list_repo_target_summaries(
        &self,
    ) -> anyhow::Result<Vec<synforge_core::api::RepoTargetSummary>>;

    async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;
}

#[async_trait]
pub trait RepoSigningConfigLoader {
    async fn load_effective_daemon_config(&self) -> anyhow::Result<DaemonConfig>;

    async fn sync_keyring_from_runtime_settings(
        &self,
        config: &mut DaemonConfig,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RepoSigningInspector {
    async fn inspect_repo_signing_status(
        &self,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoSigningStatusView>;
}

#[async_trait]
pub trait RepoSigningProgressReader {
    async fn load_repo_signing_reconcile_progress(
        &self,
    ) -> Option<RepoSigningReconcileProgressView>;
}

pub async fn get_repo_inventory<D>(
    deps: &D,
    limit: Option<usize>,
    offset: Option<usize>,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    kind: Option<ArtifactKind>,
) -> anyhow::Result<RepoInventoryResponse>
where
    D: RepoInventoryReader + Send + Sync,
{
    let (limit, offset) = normalize_pagination(limit, offset);
    let total = deps
        .count_published_repo_files(package_name.clone(), mock_chroot.clone(), kind)
        .await?;
    let repo_files = deps
        .list_published_repo_files(limit, offset, package_name, mock_chroot, kind)
        .await?;
    Ok(RepoInventoryResponse {
        page: build_page_info(limit, offset, total, repo_files.len()),
        repo_files,
    })
}

pub async fn get_repo_summary<D>(deps: &D) -> anyhow::Result<RepoSummaryResponse>
where
    D: RepoSummaryReader + Send + Sync,
{
    let (package_count, target_count, build_count) = deps.get_repo_distinct_counts().await?;
    Ok(RepoSummaryResponse {
        package_count,
        target_count,
        build_count,
        stored_bytes: deps.sum_published_repo_file_bytes().await?,
        published_file_count: deps.count_all_published_repo_files().await?,
        targets: deps.list_repo_target_summaries().await?,
        recent_files: deps.list_recent_published_repo_files(10).await?,
    })
}

pub async fn resolve_repo_file_path(
    repo_root: &Path,
    relative_repo_path: &str,
) -> anyhow::Result<PathBuf> {
    let requested = normalize_repo_path(relative_repo_path)?;
    let path = repo_root.join(&requested);
    if !tokio::fs::try_exists(&path).await? {
        return Err(anyhow::anyhow!(SynforgeError::NotFound(requested)));
    }

    let repo_root = tokio::fs::canonicalize(repo_root).await?;
    let resolved = tokio::fs::canonicalize(&path).await?;
    if !resolved.starts_with(&repo_root) {
        anyhow::bail!("resolved repo path escapes repository root");
    }
    Ok(resolved)
}

pub async fn get_repo_signing_status<D>(
    deps: &D,
    can_export_private_key: bool,
) -> anyhow::Result<RepoSigningStatusResponse>
where
    D: RepoSigningConfigLoader + RepoSigningInspector + Send + Sync,
{
    let mut config = deps.load_effective_daemon_config().await?;
    deps.sync_keyring_from_runtime_settings(&mut config).await?;
    let mut status = deps.inspect_repo_signing_status(&config).await?;
    status.can_export_private_key = can_export_private_key;
    Ok(RepoSigningStatusResponse { status })
}

pub async fn get_repo_signing_reconcile_progress<D>(
    deps: &D,
) -> anyhow::Result<RepoSigningReconcileProgressResponse>
where
    D: RepoSigningProgressReader + Send + Sync,
{
    Ok(RepoSigningReconcileProgressResponse {
        operation: deps.load_repo_signing_reconcile_progress().await,
    })
}

fn normalize_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    (
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    )
}

fn build_page_info(limit: usize, offset: usize, total: u64, returned: usize) -> PageInfo {
    PageInfo {
        limit,
        offset,
        returned,
        total: Some(total),
        has_more: (offset as u64) + (returned as u64) < total,
    }
}

fn normalize_repo_path(path: &str) -> anyhow::Result<String> {
    let trimmed = path.trim_start_matches('/');
    let normalized = PathBuf::from(trimmed);
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("repository path must not be empty");
    }
    if normalized
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        anyhow::bail!("repository path contains invalid components");
    }
    Ok(normalized.to_string_lossy().into_owned())
}
