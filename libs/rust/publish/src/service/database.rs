use async_trait::async_trait;
use synforge_core::{
    api::RepoTargetSummary,
    model::{ArtifactKind, PublishedRepoFile},
};
use synforge_database::repo::PostgresRepoStore;

use super::{RepoInventoryReader, RepoSummaryReader};

#[async_trait]
impl RepoInventoryReader for PostgresRepoStore {
    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64> {
        PostgresRepoStore::count_published_repo_files(self, package_name, mock_chroot, kind).await
    }

    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        PostgresRepoStore::list_published_repo_files(
            self,
            limit,
            offset,
            package_name,
            mock_chroot,
            kind,
        )
        .await
    }
}

#[async_trait]
impl RepoSummaryReader for PostgresRepoStore {
    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)> {
        PostgresRepoStore::get_repo_distinct_counts(self).await
    }

    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64> {
        PostgresRepoStore::sum_published_repo_file_bytes(self).await
    }

    async fn count_all_published_repo_files(&self) -> anyhow::Result<u64> {
        PostgresRepoStore::count_all_published_repo_files(self).await
    }

    async fn list_repo_target_summaries(&self) -> anyhow::Result<Vec<RepoTargetSummary>> {
        PostgresRepoStore::list_repo_target_summaries(self).await
    }

    async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        PostgresRepoStore::list_recent_published_repo_files(self, limit).await
    }
}
