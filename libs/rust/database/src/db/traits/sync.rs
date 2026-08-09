use async_trait::async_trait;
use synforge_core::sync::{
    SyncEventLevel, SyncOperation, SyncOperationEvent, SyncStage, SyncStatus, SyncTriggerType,
};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewSyncRun {
    pub package_name: String,
    pub trigger_type: SyncTriggerType,
    pub target_mock_chroot: Option<String>,
    pub batch_id: Option<Uuid>,
    pub retry_of: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EnqueuedSyncRun {
    pub operation: SyncOperation,
    /// False means an already queued/running operation for this package was
    /// returned instead of creating duplicate work.
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct SyncRunCompletion {
    pub status: SyncStatus,
    pub revision: Option<String>,
    pub previous_revision: Option<String>,
    pub changed: Option<bool>,
    pub queued_targets: u64,
    pub skipped_targets: u64,
    pub blocked_targets: u64,
    pub error_message: Option<String>,
}

#[async_trait]
pub trait SyncStore: Send + Sync {
    async fn enqueue_sync_run(&self, request: NewSyncRun) -> anyhow::Result<EnqueuedSyncRun>;

    async fn get_sync_operation(&self, id: Uuid) -> anyhow::Result<Option<SyncOperation>>;

    /// Compare-and-set a queued run to running. Returns false when another
    /// worker claimed or cancelled it first.
    async fn claim_sync_run(&self, id: Uuid) -> anyhow::Result<bool>;

    /// Advance a running operation and append its user-visible timeline
    /// event atomically. Returns false after cancellation/finalization.
    async fn advance_sync_run(
        &self,
        id: Uuid,
        stage: SyncStage,
        message: &str,
    ) -> anyhow::Result<bool>;

    /// Finalize a queued/running operation once. Returns false if a racing
    /// cancellation or completion already won.
    async fn finish_sync_run(
        &self,
        id: Uuid,
        completion: SyncRunCompletion,
    ) -> anyhow::Result<bool>;

    /// Cancel queued work immediately or flag a running worker for
    /// cancellation. Returns the current operation when it exists.
    async fn request_sync_cancellation(&self, id: Uuid) -> anyhow::Result<Option<SyncOperation>>;

    async fn list_sync_operation_events(&self, id: Uuid)
    -> anyhow::Result<Vec<SyncOperationEvent>>;

    async fn append_sync_operation_event(
        &self,
        id: Uuid,
        stage: SyncStage,
        level: SyncEventLevel,
        message: &str,
    ) -> anyhow::Result<()>;

    async fn list_queued_sync_run_ids(&self) -> anyhow::Result<Vec<Uuid>>;

    /// Mark runs that were executing during a daemon exit as interrupted.
    async fn interrupt_running_sync_runs(&self, message: &str) -> anyhow::Result<u64>;

    async fn insert_sync_operation(
        &self,
        package_name: &str,
        trigger_type: SyncTriggerType,
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
    async fn last_sync_at_per_package(&self) -> anyhow::Result<Vec<(String, OffsetDateTime)>>;
}
