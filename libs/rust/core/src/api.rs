use serde::{Deserialize, Serialize};

use crate::{
    model::{BuildArtifact, BuildJob, PackageRuntimeState, PublishedRepoFile},
    package::{BuildEnvVar, PackageDefinition, SpecSource},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatePackageRequest {
    pub name: String,
    pub source: SpecSource,
    pub mock_chroots: Vec<String>,
    pub poll_interval_seconds: u64,
    pub build_timeout_seconds: u64,
    #[serde(default = "default_package_history_count")]
    pub package_history_count: u64,
    #[serde(default)]
    pub build_env: Vec<BuildEnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdatePackageRequest {
    pub source: SpecSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_srpm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mock_chroots: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_interval_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_timeout_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_history_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_env: Option<Vec<BuildEnvVar>>,
}

fn default_package_history_count() -> u64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowseRepositoryRequest {
    pub repo_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowseRepositoryResponse {
    pub repo_url: String,
    pub head_commit: String,
    pub files: Vec<String>,
    pub spec_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockChrootListResponse {
    pub chroots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RebuildRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RefreshRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageResponse {
    pub package: PackageDefinition,
    pub state: PackageRuntimeState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageListResponse {
    pub packages: Vec<PackageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PaginationQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildJobResponse {
    pub job: BuildJob,
    pub artifacts: Vec<BuildArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildJobListResponse {
    pub jobs: Vec<BuildJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PruneJobsResponse {
    pub deleted_jobs: Vec<BuildJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageBuildInventoryEntry {
    pub build: BuildJobResponse,
    pub repo_files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageBuildHistoryResponse {
    pub package_name: String,
    pub builds: Vec<PackageBuildInventoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRepoFilesResponse {
    pub package_name: String,
    pub repo_files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoInventoryResponse {
    pub repo_files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveConfigView {
    pub listen_addr: String,
    pub bearer_token: String,
    pub runtime_root: std::path::PathBuf,
    pub database_path: std::path::PathBuf,
    pub packages_dir: std::path::PathBuf,
    pub repo_dir: std::path::PathBuf,
    pub jobs_root: std::path::PathBuf,
    pub worker_image: String,
    pub max_concurrent_builds: usize,
    pub db_pool_size: u32,
    pub queue_buffer_size: usize,
    pub poller_tick_seconds: u64,
    pub worker_result_timeout_seconds: u64,
    pub worker_socket_timeout_seconds: u64,
    pub git_operation_timeout_seconds: u64,
    pub public_base_url: String,
    pub worker_listen_addr: String,
    pub worker_connect_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveConfigDto {
    pub config: EffectiveConfigView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateRuntimeSettingsRequest {
    pub public_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogChunkResponse {
    pub job_id: uuid::Uuid,
    pub source: String,
    pub contents: String,
    pub start_line: u64,
    pub cursor: u64,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LogChunkQuery {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub cursor: Option<u64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogMetaResponse {
    pub job_id: uuid::Uuid,
    pub source: String,
    pub file_size: u64,
    pub max_cursor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogSource {
    pub name: String,
    pub path: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub source_type: LogSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogSourceType {
    Structured,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogManifestResponse {
    pub job_id: uuid::Uuid,
    pub sources: Vec<LogSource>,
}
