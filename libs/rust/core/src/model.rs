use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildTrigger {
    Poll,
    ManualRefresh,
    ManualRebuild,
    Api,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildArtifact {
    pub package_name: String,
    pub mock_chroot: String,
    pub arch: String,
    pub path: PathBuf,
    pub relative_repo_path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: ArtifactKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishedRepoFile {
    pub job_id: Uuid,
    pub package_name: String,
    pub mock_chroot: String,
    pub arch: String,
    pub repo_path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: ArtifactKind,
    pub published_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Rpm,
    Srpm,
    Log,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildJob {
    pub id: Uuid,
    pub package_name: String,
    pub mock_chroot: String,
    pub revision: String,
    pub trigger: BuildTrigger,
    pub status: BuildStatus,
    pub spec_path: PathBuf,
    pub worker_container_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageRuntimeState {
    pub last_revision: Option<String>,
    pub last_successful_build_id: Option<Uuid>,
    pub active_job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRevision {
    pub source_name: String,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerJobPayload {
    pub job_id: Uuid,
    pub workspace_dir: PathBuf,
    pub artifact_dir: PathBuf,
    pub timeout_seconds: u64,
    pub action: WorkerAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerAction {
    Parse(WorkerParsePayload),
    Build(WorkerBuildPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerParsePayload {
    pub package_name: String,
    pub source: crate::SpecSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerBuildPayload {
    pub package_name: String,
    pub package: crate::PackageDefinition,
    pub mock_chroot: String,
    pub trigger: BuildTrigger,
    pub revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerParseResult {
    pub parsed: crate::ParsedSpec,
    pub revision: crate::SpecRevision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerBuildResult {
    pub job_id: Uuid,
    pub package_name: String,
    pub status: BuildStatus,
    pub artifacts: Vec<BuildArtifact>,
    pub logs_path: Option<PathBuf>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerResult {
    Parse(WorkerParseResult),
    Build(WorkerBuildResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoPublication {
    pub package_name: String,
    pub repo_root: PathBuf,
    pub published_at: OffsetDateTime,
    pub files: Vec<PublishedRepoFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveConfigResponse {
    pub config: crate::DaemonConfig,
}

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_timestamp(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

pub fn env_map_to_vec(map: &BTreeMap<String, String>) -> Vec<(String, String)> {
    map.iter().map(|(key, value)| (key.clone(), value.clone())).collect()
}
