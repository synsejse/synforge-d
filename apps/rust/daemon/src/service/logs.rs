use uuid::Uuid;

use super::SynforgeService;
use synforge_core::{
    api::{LogManifestResponse, LogSource, LogSourceType},
    error::SynforgeError,
    model::BuildStatus,
};
use synforge_database::JobStore;
use synforge_worker_host::LogBroadcaster;

impl SynforgeService {
    /// Live log broadcaster handle used by the SSE streaming endpoint.
    pub fn log_broadcaster(&self) -> LogBroadcaster {
        self.log_broadcaster.clone()
    }

    /// Resolve the on-disk path for a job's log source, erroring with
    /// `NotFound` if the source is unknown or the file is missing.
    pub async fn resolve_job_log_path(
        &self,
        job_id: Uuid,
        source: &str,
    ) -> anyhow::Result<std::path::PathBuf> {
        let row = self
            .store
            .get_build_log_for_job_source(job_id, source)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(format!(
                    "log source {} for job {}",
                    source, job_id
                )))
            })?;
        let path = self.config.runtime_paths().job_log_path(job_id, &row.file);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
        Err(anyhow::anyhow!(SynforgeError::NotFound(
            path.display().to_string()
        )))
    }

    /// True when the job exists and has reached a terminal status (no more log
    /// output will be produced). Errors with `NotFound` if the job is unknown.
    pub async fn job_is_terminal(&self, job_id: Uuid) -> anyhow::Result<bool> {
        let job =
            self.store.get_job(job_id).await?.ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(format!("job {}", job_id)))
            })?;
        Ok(matches!(
            job.job.status,
            BuildStatus::Succeeded | BuildStatus::Failed | BuildStatus::TimedOut
        ))
    }

    pub async fn get_job_log_manifest(&self, job_id: Uuid) -> anyhow::Result<LogManifestResponse> {
        let mut sources = Vec::new();
        let db_logs = self.store.list_build_logs_for_job(job_id).await?;

        for row in db_logs {
            let log_path = self.config.runtime_paths().job_log_path(job_id, &row.file);
            if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                sources.push(LogSource {
                    file: row.file,
                    size: meta.len(),
                    source_type: LogSourceType::Raw,
                });
            }
        }

        Ok(LogManifestResponse { job_id, sources })
    }
}

/// Find the largest prefix length of `buffer` that ends on a UTF-8 character
/// boundary, trimming only a trailing incomplete code point. Invalid bytes in
/// the interior are preserved and handled by lossy decoding downstream.
pub(crate) fn find_utf8_boundary(buffer: &[u8]) -> usize {
    let len = buffer.len();
    if len == 0 {
        return 0;
    }

    for start in (len.saturating_sub(4)..len).rev() {
        match std::str::from_utf8(&buffer[start..]) {
            Ok(_) => return len,
            Err(error) if error.error_len().is_none() => return start + error.valid_up_to(),
            Err(_) => continue,
        }
    }

    len
}
