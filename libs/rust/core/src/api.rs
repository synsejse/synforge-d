use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use crate::{
    model::{
        BuildArtifact, BuildJob, BuildTrigger, PackageRuntimeState, PublishedRepoFile, UserAccount,
        UserPermission, UserRepoMetrics,
    },
    package::{BuildEnvVar, PackageDefinition, SpecSource},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PageInfo {
    pub limit: usize,
    pub offset: usize,
    pub returned: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreatePackageRequest {
    pub name: String,
    pub source: SpecSource,
    #[serde(default)]
    pub network_access: bool,
    pub mock_chroots: Vec<String>,
    pub poll_interval_seconds: u64,
    pub build_timeout_seconds: u64,
    #[serde(default = "default_package_history_count")]
    pub package_history_count: u64,
    #[serde(default)]
    pub build_env: Vec<BuildEnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdatePackageRequest {
    pub source: SpecSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_srpm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BrowseRepositoryRequest {
    pub repo_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BrowseRepositoryResponse {
    pub repo_url: String,
    pub head_commit: String,
    pub files: Vec<String>,
    pub spec_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MockChrootListResponse {
    pub chroots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
pub struct RebuildRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
pub struct RefreshRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackageActionDisposition {
    Queued,
    Skipped,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageActionTargetResult {
    pub package_name: String,
    pub mock_chroot: String,
    pub disposition: PackageActionDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<uuid::Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageActionResponse {
    pub package_name: String,
    pub trigger: BuildTrigger,
    pub results: Vec<PackageActionTargetResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageResponse {
    pub package: PackageDefinition,
    pub state: PackageRuntimeState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageListResponse {
    pub packages: Vec<PackageResponse>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct PaginationQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct PackageListQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildJobResponse {
    pub job: BuildJob,
    pub artifacts: Vec<BuildArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildJobListResponse {
    pub jobs: Vec<BuildJobResponse>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PruneJobsResponse {
    pub deleted_jobs: Vec<BuildJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageBuildInventoryEntry {
    pub build: BuildJobResponse,
    pub repo_files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageBuildHistoryResponse {
    pub package_name: String,
    pub builds: Vec<PackageBuildInventoryEntry>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PackageRepoFilesResponse {
    pub package_name: String,
    pub repo_files: Vec<PublishedRepoFile>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoInventoryResponse {
    pub repo_files: Vec<PublishedRepoFile>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct RepoInventoryQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mock_chroot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<crate::model::ArtifactKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct JobListQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<crate::model::BuildStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mock_chroot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserResponse {
    pub user: UserAccount,
    pub metrics: UserRepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateUserRequest {
    pub handle: String,
    pub display_name: String,
    pub password: String,
    pub permissions: Vec<UserPermission>,
    #[serde(default = "default_user_active")]
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateUserRequest {
    pub handle: String,
    pub display_name: String,
    pub permissions: Vec<UserPermission>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChangePasswordRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SessionLoginRequest {
    pub handle: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupStatusResponse {
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupAdminRequest {
    pub handle: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SetupInitializeRequest {
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
    pub admin: SetupAdminRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UserMetricsResponse {
    pub metrics: UserRepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoTargetSummary {
    pub mock_chroot: String,
    pub package_count: u64,
    pub build_count: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoSummaryResponse {
    pub package_count: u64,
    pub target_count: u64,
    pub build_count: u64,
    pub stored_bytes: u64,
    pub published_file_count: u64,
    pub targets: Vec<RepoTargetSummary>,
    pub recent_files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SessionResponse {
    pub user: UserAccount,
}

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EffectiveConfigView {
    #[schema(value_type = String)]
    pub config_path: std::path::PathBuf,
    pub bootstrap_completed: bool,
    pub listen_addr: String,
    #[schema(value_type = String)]
    pub runtime_root: std::path::PathBuf,
    pub database_url: String,
    #[schema(value_type = String)]
    pub packages_dir: std::path::PathBuf,
    #[schema(value_type = String)]
    pub repo_dir: std::path::PathBuf,
    #[schema(value_type = String)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EffectiveConfigDto {
    pub config: EffectiveConfigView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateRuntimeSettingsRequest {
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFieldType {
    String,
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ConfigFieldDescriptor {
    pub key: String,
    pub label: String,
    pub description: String,
    pub section_key: String,
    pub section_label: String,
    #[serde(rename = "type")]
    pub field_type: ConfigFieldType,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_value: Option<u64>,
    pub editable_in_setup: bool,
    pub editable_in_runtime: bool,
    pub default_value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ConfigSchemaResponse {
    pub fields: Vec<ConfigFieldDescriptor>,
}

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
    pub source: Option<String>,
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
    pub path: String,
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
