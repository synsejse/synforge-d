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
    Succeeded,
    Failed,
}

impl_text_enum!(SyncStatus {
    Succeeded => ["succeeded"],
    Failed => ["failed"],
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncOperation {
    pub id: String,
    pub package_name: String,
    pub trigger_type: SyncTriggerType,
    pub status: SyncStatus,
    pub revision: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}
