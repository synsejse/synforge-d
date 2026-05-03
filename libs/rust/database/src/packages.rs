//! Persistence adapter for package records.

use crate::{DieselStore, JobStore, PackageStore, RepoStore};
use synforge_core::{
    api::{BuildJobResponse, PackageResponse},
    model::{BuildJob, PublishedRepoFile},
    package::PackageDefinition,
};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPackageStore {
    store: DieselStore,
}

impl PostgresPackageStore {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>> {
        self.store
            .list_packages(limit, offset, search, enabled)
            .await
    }

    pub async fn count_packages(
        &self,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<u64> {
        self.store.count_packages(search, enabled).await
    }

    pub async fn list_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store.list_published_repo_files_for_job(job_id).await
    }

    pub async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        self.store.delete_job(job_id).await
    }

    pub async fn list_all_enabled_package_names(&self) -> anyhow::Result<Vec<String>> {
        const PAGE_SIZE: usize = 200;
        let mut package_names = Vec::new();
        let mut offset = 0;

        loop {
            let page = self
                .store
                .list_packages(PAGE_SIZE, offset, None, Some(true))
                .await?;
            if page.is_empty() {
                break;
            }
            offset += page.len();
            package_names.extend(page.into_iter().map(|entry| entry.package.name));
        }

        Ok(package_names)
    }

    pub async fn count_package_builds(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<u64> {
        self.store
            .count_jobs(
                None,
                Some(package_name.to_string()),
                None,
                false,
                include_deleted,
            )
            .await
    }

    pub async fn list_package_builds(
        &self,
        package_name: &str,
        limit: usize,
        offset: usize,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_jobs(
                limit,
                offset,
                None,
                Some(package_name.to_string()),
                None,
                false,
                include_deleted,
            )
            .await
    }

    pub async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.store
            .list_published_repo_files_for_package(package_name)
            .await
    }

    pub async fn find_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Option<PackageResponse>> {
        self.store.get_package(package_name).await
    }

    pub async fn upsert_package_definition(
        &self,
        package: &PackageDefinition,
    ) -> anyhow::Result<()> {
        self.store.upsert_package(package).await
    }

    pub async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        self.store
            .has_active_job_for_target(package_name, mock_chroot)
            .await
    }

    pub async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>> {
        self.store
            .get_last_successful_revision(package_name, mock_chroot)
            .await
    }

    pub async fn get_target_backoff_wait_seconds(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<u64>> {
        let Some(backoff) = self
            .store
            .get_target_build_backoff(package_name, mock_chroot)
            .await?
        else {
            return Ok(None);
        };
        let now = OffsetDateTime::now_utc();
        if backoff.next_eligible_at <= now {
            return Ok(None);
        }
        Ok(Some(
            (backoff.next_eligible_at - now).whole_seconds().max(1) as u64,
        ))
    }

    pub async fn insert_build_job(&self, job: &BuildJob) -> anyhow::Result<()> {
        self.store.insert_job(job).await
    }

    pub async fn list_jobs_for_package(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.store
            .list_jobs_for_package(package_name, include_deleted)
            .await
    }
}
