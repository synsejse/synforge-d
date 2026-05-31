#[path = "jobs/ports.rs"]
mod ports;

use std::path::PathBuf;

use anyhow::Context;

use synforge_core::{
    api::{
        BuildJobListResponse, BuildJobResponse, JobArtifactListResponse, JobArtifactMetaResponse,
        PruneJobsResponse, TimeSeriesResponse, build_page_info, normalize_pagination,
        resolve_time_range,
    },
    error::SynforgeError,
    model::{BuildStatus, format_timestamp},
};
use synforge_database::{JobStore, RepoStore};
use time::OffsetDateTime;
use uuid::Uuid;

use super::SynforgeService;
use super::sync::{SeriesBucket, bucket_succeeded_failed_events, snap_to_bucket};

impl SynforgeService {
    pub async fn resolve_job_artifact_path(
        &self,
        job_id: Uuid,
        relative_artifact_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let job = self
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        let artifact = job
            .artifacts
            .into_iter()
            .find(|artifact| artifact.file == std::path::Path::new(relative_artifact_path))
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(relative_artifact_path.to_string()))
            })?;

        let path = self
            .config
            .runtime_paths()
            .job_artifacts_dir(job_id)
            .join(artifact.storage_path());
        if !tokio::fs::try_exists(&path).await? {
            return Err(anyhow::anyhow!(SynforgeError::NotFound(
                path.display().to_string()
            )));
        }

        let artifacts_root =
            tokio::fs::canonicalize(self.config.runtime_paths().job_artifacts_dir(job_id))
                .await
                .with_context(|| format!("failed to resolve job artifact root for {}", job_id))?;
        let resolved_path = tokio::fs::canonicalize(&path)
            .await
            .with_context(|| format!("failed to resolve artifact path {}", path.display()))?;
        if !resolved_path.starts_with(&artifacts_root) {
            anyhow::bail!(
                "resolved artifact path {} escapes job artifact root {}",
                resolved_path.display(),
                artifacts_root.display()
            );
        }

        Ok(resolved_path)
    }

    pub async fn list_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        include_deleted: bool,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(
                status,
                package_name.clone(),
                mock_chroot.clone(),
                false,
                include_deleted,
            )
            .await?;
        let jobs = self
            .store
            .list_jobs(
                limit,
                offset,
                status,
                package_name,
                mock_chroot,
                false,
                include_deleted,
            )
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn list_completed_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        include_deleted: bool,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(
                status,
                package_name.clone(),
                mock_chroot.clone(),
                true,
                include_deleted,
            )
            .await?;
        let jobs = self
            .store
            .list_jobs(
                limit,
                offset,
                status,
                package_name,
                mock_chroot,
                true,
                include_deleted,
            )
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn list_active_jobs(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self
            .store
            .count_active_jobs(package_name.clone(), mock_chroot.clone())
            .await?;
        let jobs = self
            .store
            .list_active_jobs(limit, offset, package_name, mock_chroot)
            .await?;
        Ok(BuildJobListResponse {
            page: build_page_info(limit, offset, total, jobs.len()),
            jobs,
        })
    }

    pub async fn get_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    pub async fn kill_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        let job = self
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        if !matches!(job.job.status, BuildStatus::Pending | BuildStatus::Running) {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "job {} is not active",
                job_id
            ))));
        }

        let reason = "job killed by user request";
        self.worker_launcher
            .kill_job(job_id, job.job.worker_container_id.clone(), reason)
            .await?;
        // Compare-and-set finish: if the in-flight run_job finalizer wins
        // the race this is a no-op (returns false), so we don't double-write
        // the row or its backoff state.
        let _finalized = self
            .store
            .finish_job(job_id, BuildStatus::Failed, Some(reason), &[], &[], &[])
            .await?;
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    pub async fn retry_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        self.build_service
            .retry_job(&self.job_retry_deps(), job_id)
            .await
    }

    pub async fn get_job_artifacts(
        &self,
        job_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<JobArtifactListResponse> {
        // Ensure the job exists so a missing job maps to 404 rather than an
        // empty page.
        self.get_job(job_id).await?;
        let (limit, offset) = normalize_pagination(limit, offset);
        let total = self.store.count_job_artifacts(job_id).await?;
        let artifacts = self.store.list_job_artifacts(job_id, limit, offset).await?;
        Ok(JobArtifactListResponse {
            page: build_page_info(limit, offset, total, artifacts.len()),
            job_id,
            artifacts,
        })
    }

    pub async fn get_job_artifact_meta(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<JobArtifactMetaResponse> {
        let job = self.get_job(job_id).await?;
        let artifact = job
            .artifacts
            .into_iter()
            .find(|artifact| artifact.file == std::path::Path::new(file))
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(file.to_string())))?;
        Ok(JobArtifactMetaResponse { job_id, artifact })
    }

    pub async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        let deleted = self
            .store
            .delete_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        self.lifecycle.remove_job_dir(job_id).await;
        Ok(deleted)
    }

    pub async fn prune_failed_jobs(&self) -> anyhow::Result<PruneJobsResponse> {
        let jobs = self
            .store
            .list_jobs(10_000, 0, None, None, None, false, false)
            .await?;
        let mut deleted_jobs = Vec::new();
        for job in jobs {
            if matches!(job.job.status, BuildStatus::Failed | BuildStatus::TimedOut) {
                deleted_jobs.push(self.delete_job(job.job.id).await?);
            }
        }
        Ok(PruneJobsResponse { deleted_jobs })
    }

    pub async fn get_jobs_timeseries(
        &self,
        range: Option<String>,
    ) -> anyhow::Result<TimeSeriesResponse> {
        let (_unit, bucket_seconds, window_seconds, label) = resolve_time_range(range.as_deref());
        let now = OffsetDateTime::now_utc();
        let cutoff = snap_to_bucket(
            now - time::Duration::seconds(window_seconds),
            bucket_seconds,
        );
        let events = self.store.list_recent_build_status_events(cutoff).await?;

        // Jobs have richer status enum than sync; "failed" and "timed_out"
        // both count toward the failure tally.
        let points =
            bucket_succeeded_failed_events(cutoff, now, bucket_seconds, events, |status| {
                match status {
                    "succeeded" => Some(SeriesBucket::Succeeded),
                    "failed" | "timed_out" => Some(SeriesBucket::Failed),
                    _ => None,
                }
            });

        Ok(TimeSeriesResponse {
            range: label.to_string(),
            bucket_seconds,
            started_at: format_timestamp(cutoff),
            points,
        })
    }
}
