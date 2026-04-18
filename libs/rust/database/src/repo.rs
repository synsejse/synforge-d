//! Persistence adapter for published repository records.

use crate::{DieselStore, RepoStore};
use synforge_core::model::{ArtifactKind, ArtifactSignature, PublishedRepoFile};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepoStore {
    store: DieselStore,
}

impl PostgresRepoStore {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64> {
        self.store
            .count_published_repo_files(package_name, mock_chroot, kind)
            .await
    }

    pub async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store
            .list_published_repo_files(limit, offset, package_name, mock_chroot, kind)
            .await
    }

    pub async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)> {
        self.store.get_repo_distinct_counts().await
    }

    pub async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64> {
        self.store.sum_published_repo_file_bytes().await
    }

    pub async fn count_all_published_repo_files(&self) -> anyhow::Result<u64> {
        self.store
            .count_published_repo_files(None, None, None)
            .await
    }

    pub async fn list_all_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store
            .list_published_repo_files(limit, offset, None, None, None)
            .await
    }

    pub async fn list_repo_target_summaries(
        &self,
    ) -> anyhow::Result<Vec<synforge_core::api::RepoTargetSummary>> {
        self.store.list_repo_target_summaries().await
    }

    pub async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store.list_recent_published_repo_files(limit).await
    }

    pub async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()> {
        self.store
            .update_build_artifact_metadata(artifact_id, sha256, size_bytes)
            .await
    }

    pub async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()> {
        self.store.upsert_artifact_signatures(signatures).await
    }
}
