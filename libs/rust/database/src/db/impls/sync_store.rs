use super::super::*;

#[async_trait]
impl SyncStore for DieselStore {
    async fn enqueue_sync_run(
        &self,
        request: super::super::traits::NewSyncRun,
    ) -> anyhow::Result<super::super::traits::EnqueuedSyncRun> {
        sync_lifecycle::enqueue_sync_run(self, request).await
    }

    async fn get_sync_operation(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<synforge_core::sync::SyncOperation>> {
        sync_lifecycle::get_sync_operation(self, id).await
    }

    async fn claim_sync_run(&self, id: uuid::Uuid) -> anyhow::Result<bool> {
        sync_lifecycle::claim_sync_run(self, id).await
    }

    async fn advance_sync_run(
        &self,
        id: uuid::Uuid,
        stage: synforge_core::sync::SyncStage,
        message: &str,
    ) -> anyhow::Result<bool> {
        sync_lifecycle::advance_sync_run(self, id, stage, message).await
    }

    async fn finish_sync_run(
        &self,
        id: uuid::Uuid,
        completion: super::super::traits::SyncRunCompletion,
    ) -> anyhow::Result<bool> {
        sync_lifecycle::finish_sync_run(self, id, completion).await
    }

    async fn request_sync_cancellation(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<synforge_core::sync::SyncOperation>> {
        sync_lifecycle::request_sync_cancellation(self, id).await
    }

    async fn list_sync_operation_events(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Vec<synforge_core::sync::SyncOperationEvent>> {
        sync_events::list_sync_operation_events(self, id).await
    }

    async fn append_sync_operation_event(
        &self,
        id: uuid::Uuid,
        stage: synforge_core::sync::SyncStage,
        level: synforge_core::sync::SyncEventLevel,
        message: &str,
    ) -> anyhow::Result<()> {
        sync_events::append_sync_operation_event(self, id, stage, level, message).await
    }

    async fn list_queued_sync_run_ids(&self) -> anyhow::Result<Vec<uuid::Uuid>> {
        sync_lifecycle::list_queued_sync_run_ids(self).await
    }

    async fn interrupt_running_sync_runs(&self, message: &str) -> anyhow::Result<u64> {
        sync_lifecycle::interrupt_running_sync_runs(self, message).await
    }

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
