use super::super::*;

#[async_trait]
impl JobStore for DieselStore {
    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>> {
        job::get_last_successful_revision(self, package_name, mock_chroot).await
    }

    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        job::has_active_job_for_target(self, package_name, mock_chroot).await
    }

    async fn insert_job(&self, job: &BuildJob) -> anyhow::Result<()> {
        job::insert_job(self, job).await
    }

    async fn set_job_running(
        &self,
        job_id: Uuid,
        worker_container_id: Option<&str>,
    ) -> anyhow::Result<()> {
        job::set_job_running(self, job_id, worker_container_id).await
    }

    async fn reset_job_for_retry(
        &self,
        job_id: Uuid,
        trigger: BuildTrigger,
        revision: &str,
    ) -> anyhow::Result<()> {
        job::reset_job_for_retry(self, job_id, trigger, revision).await
    }

    async fn update_build_failure_backoff(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        base_backoff_seconds: u64,
        max_backoff_seconds: u64,
    ) -> anyhow::Result<()> {
        job::update_build_failure_backoff(
            self,
            job_id,
            status,
            base_backoff_seconds,
            max_backoff_seconds,
        )
        .await
    }

    async fn get_target_build_backoff(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<BuildFailureBackoffState>> {
        job::get_target_build_backoff(self, package_name, mock_chroot).await
    }

    async fn list_target_build_backoffs(
        &self,
    ) -> anyhow::Result<Vec<(String, String, BuildFailureBackoffState)>> {
        job::list_target_build_backoffs(self).await
    }

    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
        artifact_signatures: &[ArtifactSignature],
    ) -> anyhow::Result<bool> {
        job::finish_job(
            self,
            job_id,
            status,
            error_message,
            artifacts,
            published_files,
            artifact_signatures,
        )
        .await
    }

    async fn list_build_logs_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<BuildLogRecord>> {
        job::list_build_logs_for_job(self, job_id).await
    }

    async fn upsert_build_log(&self, job_id: Uuid, file: &str) -> anyhow::Result<()> {
        job::upsert_build_log(self, job_id, file).await
    }

    async fn get_build_log_for_job_source(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<Option<BuildLogRecord>> {
        job::get_build_log_for_job_source(self, job_id, file).await
    }

    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs(
            self,
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

    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
        include_deleted: bool,
    ) -> anyhow::Result<u64> {
        job::count_jobs(
            self,
            status,
            package_name,
            mock_chroot,
            completed_only,
            include_deleted,
        )
        .await
    }

    async fn list_active_jobs(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_active_jobs(self, limit, offset, package_name, mock_chroot).await
    }

    async fn count_active_jobs(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64> {
        job::count_active_jobs(self, package_name, mock_chroot).await
    }

    async fn list_jobs_for_package(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs_for_package(self, package_name, include_deleted).await
    }

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        job::get_job(self, job_id).await
    }

    async fn count_job_artifacts(&self, job_id: Uuid) -> anyhow::Result<u64> {
        job::count_job_artifacts(self, job_id).await
    }

    async fn list_job_artifacts(
        &self,
        job_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<BuildArtifact>> {
        job::list_job_artifacts(self, job_id, limit, offset).await
    }

    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        job::soft_delete_job(self, job_id).await
    }

    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()> {
        job::abort_unfinished_jobs(self, message).await
    }

    async fn list_prunable_successful_job_ids(
        &self,
        package_name: &str,
        mock_chroot: &str,
        keep: usize,
    ) -> anyhow::Result<Vec<Uuid>> {
        job::list_prunable_successful_job_ids(self, package_name, mock_chroot, keep).await
    }

    async fn list_recent_build_status_events(
        &self,
        cutoff: time::OffsetDateTime,
    ) -> anyhow::Result<Vec<(time::OffsetDateTime, String)>> {
        job::list_recent_build_status_events(self, cutoff).await
    }
}
