use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    model::{BuildTrigger, PackageRuntimeState},
    package::{BuildEnvVar, PackageDefinition, SpecSource},
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_limit_millicores: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit_mb: Option<u64>,
    #[serde(default)]
    pub ccache_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccache_max_size_mb: Option<u64>,
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
    pub cpu_limit_millicores: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccache_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccache_max_size_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_env: Option<Vec<BuildEnvVar>>,
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
    pub page: super::PageInfo,
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
    pub page: super::PageInfo,
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
