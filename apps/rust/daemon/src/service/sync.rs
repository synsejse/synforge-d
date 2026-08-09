use std::collections::HashMap;

use synforge_core::{
    api::{
        SyncBatchDetailResponse, SyncBatchListResponse, SyncMetricsResponse,
        SyncOperationDetailResponse, SyncOperationListResponse, SyncScheduleEntry,
        SyncScheduleResponse, TimeSeriesPoint, TimeSeriesResponse, build_page_info,
        resolve_time_range,
    },
    error::SynforgeError,
    model::format_timestamp,
    sync::SyncStatus,
};
use time::OffsetDateTime;

use super::SynforgeService;
use synforge_database::{JobStore, PackageStore, SyncStore};

impl SynforgeService {
    pub async fn get_sync_batch_detail(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<SyncBatchDetailResponse> {
        let batch = self
            .store
            .refresh_sync_batch(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(id.to_string())))?;
        let operations = self.store.list_sync_operations_for_batch(id).await?;
        Ok(SyncBatchDetailResponse { batch, operations })
    }

    pub async fn get_latest_sync_batch_detail(
        &self,
    ) -> anyhow::Result<Option<SyncBatchDetailResponse>> {
        let Some(batch) = self.store.get_latest_sync_batch().await? else {
            return Ok(None);
        };
        self.get_sync_batch_detail(uuid::Uuid::parse_str(&batch.id)?)
            .await
            .map(Some)
    }

    pub async fn list_sync_batches(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<SyncBatchListResponse> {
        self.store.refresh_active_sync_batches().await?;
        let limit = limit.unwrap_or(25).clamp(1, 100);
        let offset = offset.unwrap_or(0);
        let total = self.store.count_sync_batches().await?;
        let batches = self.store.list_sync_batches(limit, offset).await?;
        let returned = batches.len();
        Ok(SyncBatchListResponse {
            batches,
            page: build_page_info(limit, offset, total, returned),
        })
    }

    pub async fn get_sync_operation_detail(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<SyncOperationDetailResponse> {
        let operation = self
            .store
            .get_sync_operation(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(id.to_string())))?;
        let events = self.store.list_sync_operation_events(id).await?;
        let builds = self.store.list_jobs_for_sync(id).await?;
        Ok(SyncOperationDetailResponse {
            operation,
            events,
            builds,
        })
    }

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

    /// Compute package-level source polling eligibility. Build-target
    /// backoff is intentionally not applied here: it can skip a target after
    /// source inspection, but it must never make source polling itself late.
    pub async fn get_sync_schedule(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> anyhow::Result<SyncScheduleResponse> {
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let offset = offset.unwrap_or(0);
        let now = OffsetDateTime::now_utc();

        // Page size = total count so the filter-side join below runs over a
        // single fetch. Realistic enabled-package counts fit in one page.
        let total_packages = self.store.count_packages(None, Some(true)).await?;
        let packages = self
            .store
            .list_packages(total_packages as usize, 0, None, Some(true))
            .await?;

        let last_sync: HashMap<String, OffsetDateTime> = self
            .store
            .last_sync_at_per_package()
            .await?
            .into_iter()
            .collect();

        let mut entries: Vec<SyncScheduleEntry> = Vec::new();
        for response in packages {
            let pkg = &response.package;
            if !pkg.enabled || !pkg.source.poll {
                continue;
            }
            let interval = time::Duration::seconds(pkg.poll_interval_seconds.max(1) as i64);
            // Without a recorded sync we treat the package as eligible
            // immediately (next_at = now).
            let interval_eligible_at = last_sync
                .get(&pkg.name)
                .map(|last| *last + interval)
                .unwrap_or(now);

            entries.push(SyncScheduleEntry {
                package_name: pkg.name.clone(),
                next_eligible_at: format_timestamp(interval_eligible_at),
                seconds_until: (interval_eligible_at - now).whole_seconds(),
            });
        }

        entries.sort_by_key(|entry| entry.seconds_until);
        let total = entries.len() as u64;
        let items: Vec<SyncScheduleEntry> = entries.into_iter().skip(offset).take(limit).collect();
        let page = build_page_info(limit, offset, total, items.len());

        Ok(SyncScheduleResponse {
            items,
            computed_at: format_timestamp(now),
            page,
        })
    }

    pub async fn get_sync_timeseries(
        &self,
        range: Option<String>,
    ) -> anyhow::Result<TimeSeriesResponse> {
        let (_unit, bucket_seconds, window_seconds, label) = resolve_time_range(range.as_deref());
        let now = OffsetDateTime::now_utc();
        let cutoff = snap_to_bucket(
            now - time::Duration::seconds(window_seconds),
            bucket_seconds,
        );
        let events = self.store.list_recent_sync_status_events(cutoff).await?;

        let points =
            bucket_succeeded_failed_events(cutoff, now, bucket_seconds, events, |status| {
                match status {
                    "succeeded" => Some(SeriesBucket::Succeeded),
                    "failed" => Some(SeriesBucket::Failed),
                    _ => None,
                }
            });

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
