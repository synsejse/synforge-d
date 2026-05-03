use super::super::*;

#[async_trait]
impl SyncStore for DieselStore {
    async fn insert_sync_operation(
        &self,
        package_name: &str,
        trigger_type: synforge_core::sync::SyncTriggerType,
        status: synforge_core::sync::SyncStatus,
        revision: Option<&str>,
        error_message: Option<&str>,
    ) -> anyhow::Result<()> {
        sync::insert_sync_operation(
            self,
            package_name,
            trigger_type,
            status,
            revision,
            error_message,
        )
        .await
    }

    async fn list_sync_operations(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        status: Option<synforge_core::sync::SyncStatus>,
    ) -> anyhow::Result<Vec<synforge_core::sync::SyncOperation>> {
        sync::list_sync_operations(self, limit, offset, package_name, status).await
    }

    async fn count_sync_operations(
        &self,
        package_name: Option<String>,
        status: Option<synforge_core::sync::SyncStatus>,
    ) -> anyhow::Result<u64> {
        sync::count_sync_operations(self, package_name, status).await
    }

    async fn get_sync_metrics(&self) -> anyhow::Result<(usize, usize, Option<String>)> {
        sync::get_sync_metrics(self).await
    }

    async fn list_recent_sync_status_events(
        &self,
        cutoff: time::OffsetDateTime,
    ) -> anyhow::Result<Vec<(time::OffsetDateTime, String)>> {
        sync::list_recent_sync_status_events(self, cutoff).await
    }

    async fn last_sync_at_per_package(
        &self,
    ) -> anyhow::Result<Vec<(String, time::OffsetDateTime)>> {
        sync::last_sync_at_per_package(self).await
    }
}
