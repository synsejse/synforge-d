use std::sync::Arc;

use synforge_core::api::{JobResourceUsageListResponse, JobResourceUsageResponse};
use synforge_worker_host::JobUsageService;
use tokio::sync::watch;
use uuid::Uuid;

use super::SynforgeService;

impl SynforgeService {
    pub async fn list_job_resource_usage(&self) -> anyhow::Result<JobResourceUsageListResponse> {
        self.job_usage_service().list_job_resource_usage().await
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
