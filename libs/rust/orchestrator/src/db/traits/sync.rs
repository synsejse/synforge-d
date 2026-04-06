use async_trait::async_trait;
use synforge_core::sync::{SyncOperation, SyncStatus};

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
}
