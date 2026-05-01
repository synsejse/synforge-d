use synforge_core::{
    api::{
        SyncMetricsResponse, SyncOperationListResponse, TimeSeriesPoint, TimeSeriesResponse,
        build_page_info, resolve_time_range,
    },
    model::format_timestamp,
    sync::SyncStatus,
};
use time::OffsetDateTime;

use super::SynforgeService;
use synforge_database::SyncStore;

impl SynforgeService {
    pub async fn list_sync_operations(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        package_name: Option<String>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<SyncOperationListResponse> {
        let limit = limit.unwrap_or(50).clamp(1, 100);
        let offset = offset.unwrap_or(0);
        let total = self
            .store
            .count_sync_operations(package_name.clone(), status)
            .await?;
        let operations = self
            .store
            .list_sync_operations(limit, offset, package_name, status)
            .await?;
        let returned = operations.len();
        Ok(SyncOperationListResponse {
            operations,
            page: build_page_info(limit, offset, total, returned),
        })
    }

    pub async fn list_package_sync_operations(
        &self,
        package_name: &str,
        limit: Option<usize>,
        offset: Option<usize>,
        status: Option<SyncStatus>,
    ) -> anyhow::Result<SyncOperationListResponse> {
        self.registry.get_package(package_name).await?;
        self.list_sync_operations(limit, offset, Some(package_name.to_string()), status)
            .await
    }

    pub async fn get_sync_metrics(&self) -> anyhow::Result<SyncMetricsResponse> {
        let (succeeded_24h, failed_24h, last_failure_at) = self.store.get_sync_metrics().await?;
        Ok(SyncMetricsResponse {
            succeeded_24h,
            failed_24h,
            last_failure_at,
        })
    }

    pub async fn get_sync_timeseries(
        &self,
        range: Option<String>,
    ) -> anyhow::Result<TimeSeriesResponse> {
        let (_unit, bucket_seconds, window_seconds, label) =
            resolve_time_range(range.as_deref());
        let now = OffsetDateTime::now_utc();
        let cutoff = snap_to_bucket(now - time::Duration::seconds(window_seconds), bucket_seconds);
        let events = self.store.list_recent_sync_status_events(cutoff).await?;

        let points = bucket_succeeded_failed_events(
            cutoff,
            now,
            bucket_seconds,
            events,
            |status| match status {
                "succeeded" => Some(SeriesBucket::Succeeded),
                "failed" => Some(SeriesBucket::Failed),
                _ => None,
            },
        );

        Ok(TimeSeriesResponse {
            range: label.to_string(),
            bucket_seconds,
            started_at: format_timestamp(cutoff),
            points,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) enum SeriesBucket {
    Succeeded,
    Failed,
}

pub(super) fn snap_to_bucket(ts: OffsetDateTime, bucket_seconds: u64) -> OffsetDateTime {
    let epoch = ts.unix_timestamp();
    let bucket = bucket_seconds as i64;
    let snapped = epoch - epoch.rem_euclid(bucket);
    OffsetDateTime::from_unix_timestamp(snapped).unwrap_or(ts)
}

/// Folds raw `(timestamp, status)` events into a dense time-ordered
/// series of `TimeSeriesPoint`s. The closure classifies each status
/// into the succeeded or failed lane (or skips it). Empty buckets are
/// filled with zeros so the chart x-axis stays linear.
pub(super) fn bucket_succeeded_failed_events<F>(
    started_at: OffsetDateTime,
    now: OffsetDateTime,
    bucket_seconds: u64,
    events: Vec<(OffsetDateTime, String)>,
    classify: F,
) -> Vec<TimeSeriesPoint>
where
    F: Fn(&str) -> Option<SeriesBucket>,
{
    let bucket = bucket_seconds as i64;
    let mut series: Vec<TimeSeriesPoint> = Vec::new();
    let mut cursor = started_at;
    while cursor <= now {
        series.push(TimeSeriesPoint {
            timestamp: format_timestamp(cursor),
            succeeded: 0,
            failed: 0,
        });
        cursor += time::Duration::seconds(bucket);
    }

    let started_epoch = started_at.unix_timestamp();
    for (ts, status) in events {
        let Some(lane) = classify(status.as_str()) else {
            continue;
        };
        let elapsed = ts.unix_timestamp() - started_epoch;
        if elapsed < 0 {
            continue;
        }
        let idx = (elapsed / bucket) as usize;
        if let Some(point) = series.get_mut(idx) {
            match lane {
                SeriesBucket::Succeeded => point.succeeded += 1,
                SeriesBucket::Failed => point.failed += 1,
            }
        }
    }

    series
}
