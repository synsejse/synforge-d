use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::model::PublishedRepoFile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RepoInventoryResponse {
    pub repo_files: Vec<PublishedRepoFile>,
    pub page: super::PageInfo,
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
