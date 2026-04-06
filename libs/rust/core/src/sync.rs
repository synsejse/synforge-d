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
}

impl std::fmt::Display for SyncTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SyncTriggerType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "poll" => Ok(Self::Poll),
            "manual_refresh" => Ok(Self::ManualRefresh),
            "manual_rebuild" => Ok(Self::ManualRebuild),
            _ => Err(()),
        }
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
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SyncStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(()),
        }
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
