use synforge_core::{
    api::{SyncMetricsResponse, SyncOperationListResponse, build_page_info},
    sync::SyncStatus,
};

use super::SynforgeService;
use synforge_database::SyncStore;

impl SynforgeService {
    pub async fn list_sync_operations(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<SyncOperationListResponse> {
        let limit = limit.unwrap_or(50).clamp(1, 100);
        let offset = offset.unwrap_or(0);
        let total = self
            .store
            .count_sync_operations(package_name.clone(), status)
            .await?;
        let operations = self
            .store
            .list_sync_operations(limit, offset, package_name, status)
            .await?;
        let returned = operations.len();
        Ok(SyncOperationListResponse {
            operations,
            page: build_page_info(limit, offset, total, returned),
        })
    }

    pub async fn list_package_sync_operations(
        &self,
        package_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<SyncOperationListResponse> {
        self.registry.get_package(package_name).await?;
        self.list_sync_operations(limit, offset, Some(package_name.to_string()), status)
            .await
    }

    pub async fn get_sync_metrics(&self) -> anyhow::Result<SyncMetricsResponse> {
        let (succeeded_24h, failed_24h, last_failure_at) = self.store.get_sync_metrics().await?;
        Ok(SyncMetricsResponse {
            succeeded_24h,
            failed_24h,
            last_failure_at,
        })
    }
}
