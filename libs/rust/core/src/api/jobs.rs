use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::model::{BuildArtifact, BuildJob, PublishedRepoFile};

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
    pub page: super::PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PruneJobsResponse {
    pub deleted_jobs: Vec<BuildJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct JobResourceUsageSample {
    pub job_id: uuid::Uuid,
    pub container_id: String,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct JobResourceUsageListResponse {
    pub samples: Vec<JobResourceUsageSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct JobResourceUsageResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<JobResourceUsageSample>,
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
    pub page: super::PageInfo,
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
    pub scope: Option<JobListScope>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobListScope {
    All,
    Active,
    Completed,
}
