//! Persistence adapter for jobs and build records.

use crate::{DieselStore, JobStore, RepoStore};
use synforge_core::{
    api::BuildJobResponse,
    model::{BuildStatus, BuildTrigger, PublishedRepoFile},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresJobStore {
    store: DieselStore,
}

impl PostgresJobStore {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_jobs(
                limit,
                offset,
                status,
                package_name,
                mock_chroot,
                completed_only,
                include_deleted,
            )
            .await
    }

    pub async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
    ) -> anyhow::Result<u64> {
        self.store
            .count_jobs(
                status,
                package_name,
                mock_chroot,
                completed_only,
                include_deleted,
            )
            .await
    }

    pub async fn list_active_jobs(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_active_jobs(limit, offset, package_name, mock_chroot)
            .await
    }

    pub async fn count_active_jobs(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64> {
        self.store
            .count_active_jobs(package_name, mock_chroot)
            .await
    }

    pub async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        self.store.get_job(job_id).await
    }

    pub async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        self.store.delete_job(job_id).await
    }

    pub async fn reset_job_for_retry(
        &self,
        job_id: Uuid,
        trigger: BuildTrigger,
        revision: &str,
    ) -> anyhow::Result<()> {
        self.store
            .reset_job_for_retry(job_id, trigger, revision)
            .await
    }

    pub async fn list_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store.list_published_repo_files_for_job(job_id).await
    }
}
