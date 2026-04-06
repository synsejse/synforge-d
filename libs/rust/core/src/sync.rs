use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncTriggerType {
    Poll,
    ManualRefresh,
    ManualRebuild,
}

impl SyncTriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Poll => "poll",
            Self::ManualRefresh => "manual_refresh",
            Self::ManualRebuild => "manual_rebuild",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "poll" => Some(Self::Poll),
            "manual_refresh" => Some(Self::ManualRefresh),
            "manual_rebuild" => Some(Self::ManualRebuild),
            _ => None,
        }
    }
}

impl std::fmt::Display for SyncTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Succeeded,
    Failed,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
