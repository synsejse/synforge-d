use std::collections::HashMap;

use synforge_core::{
    api::{
        SyncMetricsResponse, SyncOperationListResponse, SyncScheduleEntry, SyncScheduleResponse,
        TimeSeriesPoint, TimeSeriesResponse, build_page_info, resolve_time_range,
    },
    model::format_timestamp,
    sync::SyncStatus,
};
use time::OffsetDateTime;

use super::SynforgeService;
use synforge_database::{JobStore, PackageStore, SyncStore};

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

    /// Compute the next-up poll schedule across all enabled+polling
    /// packages. Each (package, mock_chroot) pair gets a row with
    /// `next_eligible_at` derived from the most recent sync attempt
    /// + the package's poll_interval, capped against any active build
    /// failure backoff. Items are sorted soonest-first; pass `limit` to
    /// cap the response (default 20, max 100).
    pub async fn get_sync_schedule(
        &self,
        limit: Option<usize>,
    ) -> anyhow::Result<SyncScheduleResponse> {
        let limit = limit.unwrap_or(20).clamp(1, 100);
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

        let backoffs: HashMap<(String, String), (u32, OffsetDateTime)> = self
            .store
            .list_target_build_backoffs()
            .await?
            .into_iter()
            .map(|(pkg, chroot, state)| {
                ((pkg, chroot), (state.consecutive_failures, state.next_eligible_at))
            })
            .collect();

        let mut entries: Vec<SyncScheduleEntry> = Vec::new();
        for response in packages {
            let pkg = &response.package;
            if !pkg.enabled || !pkg.source.poll {
                continue;
            }
            let interval =
                time::Duration::seconds(pkg.poll_interval_seconds.max(1) as i64);
            // Without a recorded sync we treat the package as eligible
            // immediately (next_at = now).
            let interval_eligible_at = last_sync
                .get(&pkg.name)
                .map(|last| *last + interval)
                .unwrap_or(now);

            for chroot in &pkg.mock_chroots {
                let backoff = backoffs.get(&(pkg.name.clone(), chroot.clone())).copied();
                let (next_at, blocked_by_backoff, consecutive_failures) = match backoff {
                    Some((failures, backoff_until)) if backoff_until > interval_eligible_at => {
                        (backoff_until, backoff_until > now, failures)
                    }
                    Some((failures, _)) => (interval_eligible_at, false, failures),
                    None => (interval_eligible_at, false, 0),
                };

                entries.push(SyncScheduleEntry {
                    package_name: pkg.name.clone(),
                    mock_chroot: chroot.clone(),
                    next_eligible_at: format_timestamp(next_at),
                    seconds_until: (next_at - now).whole_seconds(),
                    blocked_by_backoff,
                    consecutive_failures,
                });
            }
        }

        entries.sort_by_key(|entry| entry.seconds_until);
        entries.truncate(limit);

        Ok(SyncScheduleResponse {
            items: entries,
            computed_at: format_timestamp(now),
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
