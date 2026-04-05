use async_trait::async_trait;
use synforge_core::{
    api::BuildJobResponse,
    model::{BuildArtifact, BuildJob, BuildStatus, PublishedRepoFile},
};
use uuid::Uuid;

use super::super::BuildLogRecord;

#[async_trait]
pub trait JobStore: Send + Sync {
    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>>;

    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool>;

    async fn insert_job(&self, job: &BuildJob) -> anyhow::Result<()>;

    async fn set_job_running(
        &self,
        job_id: Uuid,
        worker_container_id: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
    ) -> anyhow::Result<()>;

    async fn upsert_build_log(&self, job_id: Uuid, file: &str) -> anyhow::Result<()>;
    async fn list_build_logs_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<BuildLogRecord>>;
    async fn get_build_log_for_job_source(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<Option<BuildLogRecord>>;

    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
    ) -> anyhow::Result<u64>;

    async fn list_active_jobs(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn count_active_jobs(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64>;

    async fn list_jobs_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()>;

    async fn list_prunable_successful_job_ids(
        &self,
        package_name: &str,
        mock_chroot: &str,
        keep: usize,
    ) -> anyhow::Result<Vec<Uuid>>;
}
