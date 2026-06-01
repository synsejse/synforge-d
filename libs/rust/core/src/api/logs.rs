use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LogSource {
    pub file: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub source_type: LogSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogSourceType {
    Structured,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LogManifestResponse {
    pub job_id: uuid::Uuid,
    pub sources: Vec<LogSource>,
}
