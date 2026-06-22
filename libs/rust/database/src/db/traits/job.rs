use async_trait::async_trait;
use synforge_core::{
    api::BuildJobResponse,
    model::{
        ArtifactSignature, BuildArtifact, BuildJob, BuildStatus, BuildTrigger, PublishedRepoFile,
    },
};
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::BuildLogRecord;

#[derive(Debug, Clone)]
pub struct BuildFailureBackoffState {
    pub consecutive_failures: u32,
    pub next_eligible_at: OffsetDateTime,
}

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

    async fn reset_job_for_retry(
        &self,
        job_id: Uuid,
        trigger: BuildTrigger,
        revision: &str,
    ) -> anyhow::Result<()>;

    async fn update_build_failure_backoff(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        base_backoff_seconds: u64,
        max_backoff_seconds: u64,
    ) -> anyhow::Result<()>;

    async fn get_target_build_backoff(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<BuildFailureBackoffState>>;

    /// Returns every active backoff row keyed by `(package, chroot)`.
    /// Powers the sync-schedule endpoint without N round-trips.
    async fn list_target_build_backoffs(
        &self,
    ) -> anyhow::Result<Vec<(String, String, BuildFailureBackoffState)>>;

    /// Finalize a job. Compare-and-set on status: only transitions a job
    /// that is still `Pending`/`Running`. Returns `true` if this call
    /// performed the finalization, `false` if the job had already been
    /// finalized by a racing writer (kill vs. in-flight run completion),
    /// in which case nothing was written.
    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
        artifact_signatures: &[ArtifactSignature],
    ) -> anyhow::Result<bool>;

    async fn upsert_build_log(&self, job_id: Uuid, file: &str) -> anyhow::Result<()>;
    async fn list_build_logs_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<BuildLogRecord>>;
    async fn get_build_log_for_job_source(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<Option<BuildLogRecord>>;

    #[allow(clippy::too_many_arguments)]
    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
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
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;

    /// Count artifacts attached to a single job (for paginated listing).
    async fn count_job_artifacts(&self, job_id: Uuid) -> anyhow::Result<u64>;

    /// List a page of artifacts attached to a single job, ordered by file
    /// name, applying `limit`/`offset`.
    async fn list_job_artifacts(
        &self,
        job_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<BuildArtifact>>;
    /// Soft-delete a finished job: drops artifacts, logs, signatures and
    /// published-file rows, sets `deleted_at = now()` on the build_jobs
    /// row. Returns the prior artifact list so the caller can clean up
    /// on-disk files. Errors if the job is still pending or running.
    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()>;

    async fn list_prunable_successful_job_ids(
        &self,
        package_name: &str,
        mock_chroot: &str,
        keep: usize,
    ) -> anyhow::Result<Vec<Uuid>>;

    /// Returns raw `(created_at, status)` rows for build jobs since
    /// `cutoff`, ordered ascending. Filtered to completion-bearing
    /// statuses (succeeded / failed / timed_out) — pending and running
    /// don't represent a historical outcome. The service layer folds
    /// these into time-bucketed counts.
    async fn list_recent_build_status_events(
        &self,
        cutoff: OffsetDateTime,
    ) -> anyhow::Result<Vec<(OffsetDateTime, String)>>;
}
