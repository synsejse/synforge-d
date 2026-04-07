use std::path::PathBuf;

use anyhow::Context;
use synforge_core::{
    api::{
        BuildJobListResponse, BuildJobResponse, JobArtifactListResponse, JobArtifactMetaResponse,
        PruneJobsResponse,
    },
    error::SynforgeError,
    model::BuildStatus,
};
use uuid::Uuid;

use super::SynforgeService;
use crate::db::{JobStore, RepoStore};

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
