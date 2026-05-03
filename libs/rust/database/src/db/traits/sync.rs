use async_trait::async_trait;
use synforge_core::sync::{SyncOperation, SyncStatus};
use time::OffsetDateTime;

#[async_trait]
pub trait SyncStore: Send + Sync {
    async fn insert_sync_operation(
        &self,
        package_name: &str,
        trigger_type: synforge_core::sync::SyncTriggerType,
        status: SyncStatus,
        revision: Option<&str>,
        error_message: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn list_sync_operations(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<Vec<SyncOperation>>;

    async fn count_sync_operations(
        &self,
        package_name: Option<String>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<u64>;

    async fn get_sync_metrics(&self) -> anyhow::Result<(usize, usize, Option<String>)>;

    /// Returns the raw `(created_at, status)` rows for every sync operation
    /// recorded since `cutoff`, ordered ascending. The service layer folds
    /// these into time-bucketed succeeded/failed counts.
    async fn list_recent_sync_status_events(
        &self,
        cutoff: OffsetDateTime,
    ) -> anyhow::Result<Vec<(OffsetDateTime, String)>>;

    /// Returns the most recent sync attempt timestamp per package, regardless
    /// of outcome. Used by the sync-schedule endpoint to compute when each
    /// target becomes eligible to poll again.
    async fn last_sync_at_per_package(
        &self,
    ) -> anyhow::Result<Vec<(String, OffsetDateTime)>>;
}
