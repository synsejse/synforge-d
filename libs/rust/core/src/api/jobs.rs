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
    pub page: super::PageInfo,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct JobResourceUsageSample {
    pub job_id: uuid::Uuid,
    pub container_id: String,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    /// Live CPU utilization as a percentage of a single core (e.g. `150.0`
    /// means ~1.5 cores busy). Computed from the delta between consecutive
    /// samples; `0.0` on the first sample for a container.
    pub cpu_percent: f64,
    /// Number of CPUs visible to the container, used to scale `cpu_percent`.
    pub online_cpus: u32,
    pub collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct JobResourceUsageListResponse {
    pub samples: Vec<JobResourceUsageSample>,
    pub page: super::PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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
    /// When true, soft-deleted jobs are included in the result. Defaults
    /// to false — soft-deleted jobs are hidden from listings; the
    /// timeseries endpoint always counts them regardless.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_deleted: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobListScope {
    All,
    Active,
    Completed,
}
