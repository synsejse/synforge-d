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
    sync::{SyncOperation, SyncStatus},
};

// --- Pagination ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PageInfo {
    pub limit: usize,
    pub offset: usize,
    pub returned: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub has_more: bool,
}

// --- Packages ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreatePackageRequest {
    pub name: String,
    pub source: SpecSource,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_publish_srpm")]
    pub publish_srpm: bool,
    #[serde(default = "default_publish_debuginfo")]
    pub publish_debuginfo: bool,
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

/// PATCH-style update payload for packages.
/// Optional fields use "None means keep existing value" semantics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdatePackageRequest {
    pub source: SpecSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_srpm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_debuginfo: Option<bool>,
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

fn default_enabled() -> bool {
    true
}

fn default_publish_srpm() -> bool {
    true
}

fn default_publish_debuginfo() -> bool {
    true
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
#[serde(rename_all = "snake_case")]
pub enum BrowseRepositoryProgressState {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BrowseRepositoryProgressView {
    pub operation_id: uuid::Uuid,
    pub repo_url: String,
    pub state: BrowseRepositoryProgressState,
    pub progress_percent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BrowseRepositoryProgressResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<BrowseRepositoryProgressView>,
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
#[serde(rename_all = "snake_case")]
pub enum RefreshAllPackagesState {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RefreshAllPackagesProgressView {
    pub operation_id: uuid::Uuid,
    pub state: RefreshAllPackagesState,
    pub total_packages: u64,
    pub processed_packages: u64,
    pub queued_packages: u64,
    pub skipped_packages: u64,
    pub blocked_packages: u64,
    pub failed_packages: u64,
    pub queued_targets: u64,
    pub skipped_targets: u64,
    pub blocked_targets: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RefreshAllPackagesProgressResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<RefreshAllPackagesProgressView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RefreshAllPackagesResponse {
    pub operation: RefreshAllPackagesProgressView,
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

// --- Jobs & Artifacts ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BuildJobResponse {
    pub job: BuildJob,
    pub artifacts: Vec<BuildArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct JobArtifactListResponse {
    pub job_id: uuid::Uuid,
    pub artifacts: Vec<BuildArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct JobArtifactMetaResponse {
    pub job_id: uuid::Uuid,
    pub artifact: BuildArtifact,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct SyncOperationListQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct PackageSyncOperationListQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncOperationListResponse {
    pub operations: Vec<SyncOperation>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncMetricsResponse {
    pub succeeded_24h: usize,
    pub failed_24h: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_failure_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MockChrootCacheStats {
    pub ttl_seconds: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub stale_served_count: u64,
    pub cached_chroot_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GitMirrorCacheStats {
    #[schema(value_type = String)]
    pub mirror_root: std::path::PathBuf,
    pub refresh_ttl_seconds: u64,
    pub max_unused_seconds: u64,
    pub tracked_mirrors: u64,
    pub stale_mirrors: u64,
    pub mirror_directories: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_fetched_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CacheStatsResponse {
    pub collected_at: String,
    pub mock_chroot_cache: MockChrootCacheStats,
    pub git_mirror_cache: GitMirrorCacheStats,
}

// --- Users, Session, Setup ---
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
    #[serde(default = "default_setup_enable_signing")]
    pub enable_signing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_armored_private_key: Option<String>,
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

fn default_setup_enable_signing() -> bool {
    true
}

// --- Common API errors ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

// --- Runtime configuration ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EffectiveConfigView {
    pub bootstrap_completed: bool,
    pub listen_addr: String,
    pub database_url: String,
    #[schema(value_type = String)]
    pub packages_dir: std::path::PathBuf,
    #[schema(value_type = String)]
    pub repo_dir: std::path::PathBuf,
    #[schema(value_type = String)]
    pub jobs_root: std::path::PathBuf,
    pub worker_image: String,
    pub signing_enabled: bool,
    pub signing_key_id: Option<String>,
    pub max_concurrent_builds: usize,
    pub db_pool_size: u32,
    pub queue_buffer_size: usize,
    pub poller_tick_seconds: u64,
    pub worker_result_timeout_seconds: u64,
    pub worker_socket_timeout_seconds: u64,
    pub git_operation_timeout_seconds: u64,
    pub public_base_url: String,
    pub mock_chroot_cache_ttl_seconds: u64,
    pub git_mirror_refresh_ttl_seconds: u64,
    pub git_mirror_max_unused_seconds: u64,
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
    Boolean,
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

// --- Repository signing ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoSigningStatusView {
    pub enabled: bool,
    pub configured_key_id: Option<String>,
    #[schema(value_type = String)]
    pub keyring_dir: std::path::PathBuf,
    #[schema(value_type = String)]
    pub repo_public_key_path: std::path::PathBuf,
    pub key_present: bool,
    pub active_fingerprint: Option<String>,
    pub can_export_private_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoSigningStatusResponse {
    pub status: RepoSigningStatusView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateRepoSigningConfigRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ImportRepoSigningKeyRequest {
    pub armored_private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ImportRepoSigningKeyResponse {
    pub key_id: String,
    pub fingerprint: String,
    pub status: RepoSigningStatusView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GenerateRepoSigningKeyResponse {
    pub key_id: String,
    pub fingerprint: String,
    pub status: RepoSigningStatusView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TestRepoSigningResponse {
    pub signed: bool,
    #[schema(value_type = String)]
    pub signature_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ExportRepoSigningKeyResponse {
    pub key_id: String,
    pub fingerprint: String,
    pub armored_private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ExportRepoSigningPublicKeyResponse {
    pub key_id: String,
    pub fingerprint: String,
    pub public_key_name: String,
    pub armored_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepoSigningReconcileMode {
    Sign,
    Unsign,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepoSigningReconcileState {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoSigningReconcileProgressView {
    pub operation_id: uuid::Uuid,
    pub mode: RepoSigningReconcileMode,
    pub state: RepoSigningReconcileState,
    pub total_artifacts: u64,
    pub processed_artifacts: u64,
    pub failed_artifacts: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoSigningReconcileProgressResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<RepoSigningReconcileProgressView>,
}

// --- Logs ---
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
