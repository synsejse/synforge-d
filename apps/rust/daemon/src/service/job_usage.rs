use std::sync::Arc;

use synforge_core::api::{
    JobResourceUsageListResponse, JobResourceUsageResponse, build_page_info, normalize_pagination,
};
use synforge_worker_host::JobUsageService;
use tokio::sync::watch;
use uuid::Uuid;

use super::SynforgeService;

impl SynforgeService {
    pub async fn list_job_resource_usage(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<JobResourceUsageListResponse> {
        let full = self.job_usage_service().list_job_resource_usage().await?;
        let total = full.len();
        let (limit, offset) = normalize_pagination(limit, offset);
        let samples: Vec<_> = full.into_iter().skip(offset).take(limit).collect();
        let page = build_page_info(limit, offset, total as u64, samples.len());
        Ok(JobResourceUsageListResponse { samples, page })
    }

    pub async fn get_job_resource_usage(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<JobResourceUsageResponse> {
        self.job_usage_service()
            .get_job_resource_usage(job_id)
            .await
    }

    pub(crate) fn start_job_usage_sampler(self: &Arc<Self>, shutdown_rx: watch::Receiver<bool>) {
        self.job_usage_service()
            .start_sampler(self.task_tracker.clone(), shutdown_rx);
    }

    fn job_usage_service(&self) -> JobUsageService {
        JobUsageService::new(self.sessions.clone(), self.runtime_cache.clone())
    }
}
