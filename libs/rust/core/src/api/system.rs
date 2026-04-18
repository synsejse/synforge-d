use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::sync::{SyncOperation, SyncStatus};

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
    pub page: super::PageInfo,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ServerHardwareResponse {
    pub cpu_cores: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_memory_mb: Option<u64>,
}
