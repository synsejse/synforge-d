use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LogChunkResponse {
    pub job_id: uuid::Uuid,
    pub source: String,
    pub contents: String,
    pub start_line: u64,
    pub cursor: u64,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams)]
pub struct LogChunkQuery {
    #[serde(default)]
    pub cursor: Option<u64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LogMetaResponse {
    pub job_id: uuid::Uuid,
    pub source: String,
    pub file_size: u64,
    pub max_cursor: u64,
}

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
