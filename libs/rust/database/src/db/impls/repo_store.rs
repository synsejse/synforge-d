use super::super::*;

#[async_trait]
impl RepoStore for DieselStore {
    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files(self, limit, offset, package_name, mock_chroot, kind).await
    }

    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64> {
        repo::count_published_repo_files(self, package_name, mock_chroot, kind).await
    }

    async fn list_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_job(self, job_id).await
    }

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_package(self, package_name).await
    }

    async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_recent_published_repo_files(self, limit).await
    }

    async fn list_repo_target_summaries(&self) -> anyhow::Result<Vec<RepoTargetSummary>> {
        repo::list_repo_target_summaries(self).await
    }

    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)> {
        repo::get_repo_distinct_counts(self).await
    }

    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64> {
        repo::sum_published_repo_file_bytes(self).await
    }
}
