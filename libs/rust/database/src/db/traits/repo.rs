use async_trait::async_trait;
use synforge_core::{
    api::RepoTargetSummary,
    model::{ArtifactKind, PublishedRepoFile},
};
use uuid::Uuid;

#[async_trait]
pub trait RepoStore: Send + Sync {
    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64>;

    async fn list_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;

    async fn list_repo_target_summaries(&self) -> anyhow::Result<Vec<RepoTargetSummary>>;
    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)>;
    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64>;
}
