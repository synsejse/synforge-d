use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::text_enum::impl_text_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncTriggerType {
    Poll,
    ManualRefresh,
    ManualRebuild,
}

impl_text_enum!(SyncTriggerType {
    Poll => ["poll"],
    ManualRefresh => ["manual_refresh"],
    ManualRebuild => ["manual_rebuild"],
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
}

impl_text_enum!(SyncStatus {
    Queued => ["queued"],
    Running => ["running"],
    Succeeded => ["succeeded"],
    Failed => ["failed"],
    Cancelled => ["cancelled"],
    Interrupted => ["interrupted"],
});

impl SyncStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Interrupted
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncStage {
    Queued,
    InspectingSource,
    UpdatingPackage,
    PlanningBuilds,
    EnqueuingBuilds,
    Completed,
}

impl_text_enum!(SyncStage {
    Queued => ["queued"],
    InspectingSource => ["inspecting_source"],
    UpdatingPackage => ["updating_package"],
    PlanningBuilds => ["planning_builds"],
    EnqueuingBuilds => ["enqueuing_builds"],
    Completed => ["completed"],
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncEventLevel {
    Info,
    Warning,
    Error,
}

impl_text_enum!(SyncEventLevel {
    Info => ["info"],
    Warning => ["warning", "warn"],
    Error => ["error"],
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncBatchStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
}

impl_text_enum!(SyncBatchStatus {
    Queued => ["queued"],
    Running => ["running"],
    Succeeded => ["succeeded"],
    Failed => ["failed"],
    Cancelled => ["cancelled"],
    Interrupted => ["interrupted"],
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncOperation {
    pub id: String,
    pub package_name: String,
    pub trigger_type: SyncTriggerType,
    pub status: SyncStatus,
    pub stage: SyncStage,
    pub revision: Option<String>,
    pub previous_revision: Option<String>,
    pub changed: Option<bool>,
    pub target_mock_chroot: Option<String>,
    pub batch_id: Option<String>,
    pub retry_of: Option<String>,
    pub cancellation_requested: bool,
    pub queued_targets: u64,
    pub skipped_targets: u64,
    pub blocked_targets: u64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncOperationEvent {
    pub id: String,
    pub sync_operation_id: String,
    pub stage: SyncStage,
    pub level: SyncEventLevel,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncBatch {
    pub id: String,
    pub trigger_type: SyncTriggerType,
    pub status: SyncBatchStatus,
    pub total_packages: u64,
    pub completed_packages: u64,
    pub succeeded_packages: u64,
    pub failed_packages: u64,
    pub cancelled_packages: u64,
    pub deduplicated_packages: u64,
    pub enqueue_failed_packages: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
}
