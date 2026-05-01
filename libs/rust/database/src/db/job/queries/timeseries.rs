use diesel_async::RunQueryDsl;
use synforge_core::model::BuildStatus;
use time::OffsetDateTime;

use super::*;

/// Selects raw `(created_at, status)` rows for completed build jobs since
/// `cutoff`. Filters to the three completion-bearing statuses
/// (succeeded / failed / timed_out) — pending and running rows don't
/// represent a historical outcome. Bucketing into time windows happens at
/// the service layer.
pub(in crate::db) async fn list_recent_build_status_events(
    store: &DieselStore,
    cutoff: OffsetDateTime,
) -> anyhow::Result<Vec<(OffsetDateTime, String)>> {
    let mut conn = store.get_connection().await?;
    let rows: Vec<(OffsetDateTime, BuildStatus)> = build_jobs::table
        .filter(build_jobs::created_at.gt(&cutoff))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Succeeded)
                .or(build_jobs::status.eq(BuildStatus::Failed))
                .or(build_jobs::status.eq(BuildStatus::TimedOut)),
        )
        .select((build_jobs::created_at, build_jobs::status))
        .order_by(build_jobs::created_at.asc())
        .load(&mut conn)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(ts, status)| (ts, status_label(status).to_string()))
        .collect())
}

fn status_label(status: BuildStatus) -> &'static str {
    match status {
        BuildStatus::Pending => "pending",
        BuildStatus::Running => "running",
        BuildStatus::Succeeded => "succeeded",
        BuildStatus::Failed => "failed",
        BuildStatus::TimedOut => "timed_out",
    }
}
