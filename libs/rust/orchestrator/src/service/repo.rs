use std::path::PathBuf;

use super::SynforgeService;
use crate::db::JobStore;
use synforge_core::{
    api::{RepoInventoryResponse, RepoSummaryResponse},
    error::SynforgeError,
    model::ArtifactKind,
};

impl SynforgeService {
    pub async fn get_repo_inventory(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<RepoInventoryResponse> {
        let (limit, offset) = super::normalize_pagination(limit, offset);
        let total = self
            .store
            .count_published_repo_files(package_name.clone(), mock_chroot.clone(), kind)
            .await?;
        let repo_files = self
            .store
            .list_published_repo_files(limit, offset, package_name, mock_chroot, kind)
            .await?;
        Ok(RepoInventoryResponse {
            page: super::build_page_info(limit, offset, total, repo_files.len()),
            repo_files,
        })
    }

    pub async fn get_repo_summary(&self) -> anyhow::Result<RepoSummaryResponse> {
        let (package_count, target_count, build_count) =
            self.store.get_repo_distinct_counts().await?;
        Ok(RepoSummaryResponse {
            package_count,
            target_count,
            build_count,
            stored_bytes: self.store.sum_published_repo_file_bytes().await?,
            published_file_count: self
                .store
                .count_published_repo_files(None, None, None)
                .await?,
            targets: self.store.list_repo_target_summaries().await?,
            recent_files: self.store.list_recent_published_repo_files(10).await?,
        })
    }

    pub async fn resolve_repo_file_path(
        &self,
        relative_repo_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let requested = normalize_repo_path(relative_repo_path)?;
        let repo_root = self.config.runtime_paths().repo_dir().to_path_buf();
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
