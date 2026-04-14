use std::path::PathBuf;

use anyhow::Context;
use std::path::Path;

use synforge_core::{
    api::{
        BuildJobListResponse, BuildJobResponse, JobArtifactListResponse, JobArtifactMetaResponse,
        PruneJobsResponse,
    },
    error::SynforgeError,
    model::{BuildStatus, BuildTrigger},
};
use uuid::Uuid;

use super::SynforgeService;
use synforge_runtime::{QueuedBuild, source::sync_tracker::sync_trigger_from_build_trigger};
use synforge_store::{JobStore, RepoStore};

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
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = super::normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(status, package_name.clone(), mock_chroot.clone(), false)
            .await?;
        let jobs = self
            .store
            .list_jobs(limit, offset, status, package_name, mock_chroot, false)
            .await?;
        Ok(BuildJobListResponse {
            page: super::build_page_info(limit, offset, total, jobs.len()),
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
    ) -> anyhow::Result<BuildJobListResponse> {
        let (limit, offset) = super::normalize_pagination(limit, offset);
        let total = self
            .store
            .count_jobs(status, package_name.clone(), mock_chroot.clone(), true)
            .await?;
        let jobs = self
            .store
            .list_jobs(limit, offset, status, package_name, mock_chroot, true)
            .await?;
        Ok(BuildJobListResponse {
            page: super::build_page_info(limit, offset, total, jobs.len()),
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
        let (limit, offset) = super::normalize_pagination(limit, offset);
        let total = self
            .store
            .count_active_jobs(package_name.clone(), mock_chroot.clone())
            .await?;
        let jobs = self
            .store
            .list_active_jobs(limit, offset, package_name, mock_chroot)
            .await?;
        Ok(BuildJobListResponse {
            page: super::build_page_info(limit, offset, total, jobs.len()),
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
        self.store
            .finish_job(job_id, BuildStatus::Failed, Some(reason), &[], &[], &[])
            .await?;
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    pub async fn retry_job(&self, job_id: Uuid) -> anyhow::Result<BuildJobResponse> {
        let job = self
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        if matches!(job.job.status, BuildStatus::Pending | BuildStatus::Running) {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(format!(
                "job {} is still active; use kill before retry",
                job_id
            ))));
        }

        let trigger = BuildTrigger::ManualRebuild;
        let package = self.registry.get_definition(&job.job.package_name).await?;
        let (_, revision) = self
            .registry
            .sync_existing_source_tracked(&package, sync_trigger_from_build_trigger(&trigger))
            .await?;

        if self
            .store
            .has_active_job_for_target(&job.job.package_name, &job.job.mock_chroot)
            .await?
        {
            return Err(anyhow::anyhow!(SynforgeError::Conflict(
                "retry target is already queued or running".to_string(),
            )));
        }

        let published_files = self.store.list_published_repo_files_for_job(job_id).await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        self.worker_launcher.cleanup_session(job_id);
        self.cleanup_retry_runtime_dirs(job_id).await?;
        self.store
            .reset_job_for_retry(job_id, trigger, &revision.comparison_key())
            .await?;

        self.queue_tx
            .send(QueuedBuild {
                package,
                mock_chroot: job.job.mock_chroot.clone(),
                revision,
                trigger,
                job_id,
            })
            .await
            .map_err(|error| anyhow::anyhow!("failed to queue retry build: {}", error))?;

        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))
    }

    async fn cleanup_retry_runtime_dirs(&self, job_id: Uuid) -> anyhow::Result<()> {
        let runtime_root = self.config.runtime_paths().job_root(job_id);
        remove_retry_runtime_dir(&runtime_root).await?;

        let worker_root = self.config.worker_jobs_root().join(job_id.to_string());
        if worker_root != runtime_root {
            remove_retry_runtime_dir(&worker_root).await?;
        }

        Ok(())
    }

    pub async fn get_job_artifacts(&self, job_id: Uuid) -> anyhow::Result<JobArtifactListResponse> {
        let job = self.get_job(job_id).await?;
        Ok(JobArtifactListResponse {
            job_id,
            artifacts: job.artifacts,
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
        Ok(deleted)
    }

    pub async fn prune_failed_jobs(&self) -> anyhow::Result<PruneJobsResponse> {
        let jobs = self
            .store
            .list_jobs(10_000, 0, None, None, None, false)
            .await?;
        let mut deleted_jobs = Vec::new();
        for job in jobs {
            if matches!(job.job.status, BuildStatus::Failed | BuildStatus::TimedOut) {
                deleted_jobs.push(self.delete_job(job.job.id).await?);
            }
        }
        Ok(PruneJobsResponse { deleted_jobs })
    }
}

async fn remove_retry_runtime_dir(path: &Path) -> anyhow::Result<()> {
    if !tokio::fs::try_exists(path).await? {
        return Ok(());
    }
    tokio::fs::remove_dir_all(path)
        .await
        .with_context(|| format!("failed to clean retry runtime directory {}", path.display()))
}
