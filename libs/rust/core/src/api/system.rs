use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    api::BuildJobResponse,
    sync::{SyncBatch, SyncOperation, SyncOperationEvent, SyncStatus},
};

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
pub struct SyncEnqueueResponse {
    pub operation: SyncOperation,
    /// False when this request was deduplicated onto an already active run
    /// for the same package.
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncOperationDetailResponse {
    pub operation: SyncOperation,
    pub events: Vec<SyncOperationEvent>,
    /// Builds planned by this sync run. Empty is a normal no-change result.
    pub builds: Vec<BuildJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncBatchDetailResponse {
    pub batch: SyncBatch,
    pub operations: Vec<SyncOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncBatchListResponse {
    pub batches: Vec<SyncBatch>,
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

/// One bucket on a histogram-style time series. `succeeded` and `failed`
/// counts cover the same interval starting at `timestamp` and lasting
/// `bucket_seconds` (from the parent response).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TimeSeriesPoint {
    pub timestamp: String,
    pub succeeded: u64,
    pub failed: u64,
}

/// One package-level source poll returned by `/api/v1/sync/schedule`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncScheduleEntry {
    pub package_name: String,
    /// When this package source becomes (or became) eligible to poll.
    pub next_eligible_at: String,
    /// Seconds from now until eligibility. Negative when overdue.
    pub seconds_until: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SyncScheduleResponse {
    pub items: Vec<SyncScheduleEntry>,
    /// Server clock when the schedule was computed; lets the UI tick
    /// down the countdown without drifting against the server.
    pub computed_at: String,
    pub page: super::PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct SyncScheduleQuery {
    /// Maximum number of entries to return. Defaults to 20, capped at 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Number of entries to skip before returning results. Defaults to 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TimeSeriesResponse {
    /// Echo of the requested range token: `"24h" | "7d" | "30d"`.
    pub range: String,
    /// Width of each bucket in seconds.
    pub bucket_seconds: u64,
    /// First bucket's timestamp (oldest, snapped to bucket boundary).
    pub started_at: String,
    pub points: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct TimeSeriesQuery {
    /// Window of history to bucket. One of `"24h"`, `"7d"`, `"30d"`.
    /// Defaults to `"24h"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

/// Resolves a range token into
/// `(bucket_unit, bucket_seconds, window_seconds, canonical_label)`.
/// `bucket_unit` is the precision string we hand to PostgreSQL's
/// `date_trunc` ("hour" or "day"); `bucket_seconds` is the matching
/// width used for empty-bucket fill on the service side.
/// Unknown values fall back to 24h so the frontend can't poison the daemon.
pub fn resolve_time_range(range: Option<&str>) -> (&'static str, u64, i64, &'static str) {
    match range.unwrap_or("24h") {
        "7d" => ("hour", 3_600, 7 * 24 * 3600, "7d"), // 168 hourly points
        "30d" => ("day", 86_400, 30 * 24 * 3600, "30d"), // 30 daily points
        _ => ("hour", 3_600, 24 * 3600, "24h"),       // 24 hourly points
    }
}
