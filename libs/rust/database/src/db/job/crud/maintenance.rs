use std::cmp::min;

use diesel::OptionalExtension;
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use time::Duration;

use super::super::*;

/// Soft-delete a finished job: drop artifacts, logs, signatures, and
/// published-file rows, then mark the job row with `deleted_at = now()`.
/// The row stays so historical statistics (timeseries, dashboards) keep
/// counting the build. The on-disk `state/jobs/{job_id}/` directory is
/// removed by the service layer caller, not here.
///
/// Idempotent: a second call on an already-soft-deleted row is a no-op
/// at the DB layer (the conditional UPDATE doesn't match) but still
/// returns a successful response.
///
/// Returns the (active or already-soft-deleted) job's prior artifact
/// list so the caller can clean up published files on disk.
pub(in crate::db) async fn soft_delete_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Option<BuildJobResponse>> {
    let mut conn = store.get_connection().await?;
    let row = build_jobs::table
        .find(job_id)
        .select(JobRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    let artifacts =
        helpers::load_artifacts_map_for_rows(&mut conn, row.as_ref().into_iter()).await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let response = build_job_response_from_row(row, &artifacts)?;
    if matches!(
        response.job.status,
        BuildStatus::Pending | BuildStatus::Running
    ) {
        return Err(anyhow::anyhow!("cannot delete a pending or running job"));
    }

    conn.transaction::<(), anyhow::Error, _>(|conn| {
        async move {
            diesel::delete(
                published_repo_files::table.filter(
                    published_repo_files::artifact_id.eq_any(
                        build_artifacts::table
                            .filter(build_artifacts::job_id.eq(job_id))
                            .select(build_artifacts::id),
                    ),
                ),
            )
            .execute(conn)
            .await?;
            diesel::delete(
                build_artifacts::table.filter(build_artifacts::job_id.eq(job_id)),
            )
            .execute(conn)
            .await?;
            diesel::delete(build_logs::table.filter(build_logs::job_id.eq(job_id)))
                .execute(conn)
                .await?;
            let now = now_utc();
            diesel::update(
                build_jobs::table
                    .find(job_id)
                    .filter(build_jobs::deleted_at.is_null()),
            )
            .set((
                build_jobs::deleted_at.eq(Some(now)),
                build_jobs::updated_at.eq(now),
            ))
            .execute(conn)
            .await?;
            Ok(())
        }
        .scope_boxed()
    })
    .await?;

    Ok(Some(response))
}

pub(in crate::db) async fn abort_unfinished_jobs(
    store: &DieselStore,
    message: &str,
) -> anyhow::Result<()> {
    let message = message.to_string();
    let mut conn = store.get_connection().await?;
    let now = now_utc();
    diesel::update(build_jobs::table.filter(build_jobs::finished_at.is_null()))
        .set((
            build_jobs::status.eq(BuildStatus::Failed),
            build_jobs::updated_at.eq(now),
            build_jobs::finished_at.eq(Some(now)),
            build_jobs::error_message.eq(Some(message.as_str())),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub(in crate::db) async fn upsert_build_log(
    store: &DieselStore,
    job_id: Uuid,
    file: &str,
) -> anyhow::Result<()> {
    let file = file.to_string();
    let updated_at = now_utc();
    let mut conn = store.get_connection().await?;
    let existing: Option<String> = build_logs::table
        .find((job_id, file.as_str()))
        .select(build_logs::file)
        .first(&mut conn)
        .await
        .optional()?;

    if existing.is_some() {
        diesel::update(build_logs::table.find((job_id, file.as_str())))
            .set(build_logs::updated_at.eq(updated_at))
            .execute(&mut conn)
            .await?;
    } else {
        let row = NewBuildLogRecord {
            job_id,
            file: file.as_str(),
            updated_at,
        };
        diesel::insert_into(build_logs::table)
            .values(&row)
            .execute(&mut conn)
            .await?;
    }
    Ok(())
}

pub(in crate::db) async fn update_build_failure_backoff(
    store: &DieselStore,
    job_id: Uuid,
    status: BuildStatus,
    base_backoff_seconds: u64,
    max_backoff_seconds: u64,
) -> anyhow::Result<()> {
    let base_backoff_seconds = i64::try_from(base_backoff_seconds)
        .map_err(|_| anyhow::anyhow!("base backoff seconds out of range"))?;
    let max_backoff_seconds = i64::try_from(max_backoff_seconds)
        .map_err(|_| anyhow::anyhow!("max backoff seconds out of range"))?;
    let mut conn = store.get_connection().await?;
    conn.transaction::<(), anyhow::Error, _>(|conn| {
        async move {
            let job = build_jobs::table
                .find(job_id)
                .select((build_jobs::package_name, build_jobs::mock_chroot))
                .first::<(String, String)>(conn)
                .await?;

            let now = now_utc();
            if status == BuildStatus::Succeeded {
                diesel::delete(build_failure_backoff::table.find((job.0.as_str(), job.1.as_str())))
                    .execute(conn)
                    .await?;
                return Ok(());
            }

            let should_backoff = matches!(status, BuildStatus::Failed | BuildStatus::TimedOut);
            if !should_backoff {
                return Ok(());
            }

            let existing = build_failure_backoff::table
                .find((job.0.as_str(), job.1.as_str()))
                .select(BuildFailureBackoffRecord::as_select())
                .first(conn)
                .await
                .optional()?;
            let next_failures = existing
                .as_ref()
                .map_or(1_i32, |record| {
                    record.consecutive_failures.saturating_add(1)
                })
                .clamp(1, 31);
            let exponent = (next_failures - 1) as u32;
            let backoff_seconds = min(
                base_backoff_seconds.saturating_mul(1_i64 << exponent),
                max_backoff_seconds,
            );
            let next_eligible = now + Duration::seconds(backoff_seconds);

            if existing.is_some() {
                diesel::update(build_failure_backoff::table.find((job.0.as_str(), job.1.as_str())))
                    .set((
                        build_failure_backoff::consecutive_failures.eq(next_failures),
                        build_failure_backoff::next_eligible_at.eq(next_eligible),
                        build_failure_backoff::updated_at.eq(now),
                    ))
                    .execute(conn)
                    .await?;
            } else {
                let row = NewBuildFailureBackoffRecord {
                    package_name: job.0.as_str(),
                    mock_chroot: job.1.as_str(),
                    consecutive_failures: next_failures,
                    next_eligible_at: next_eligible,
                    updated_at: now,
                };
                diesel::insert_into(build_failure_backoff::table)
                    .values(&row)
                    .execute(conn)
                    .await?;
            }
            Ok(())
        }
        .scope_boxed()
    })
    .await?;
    Ok(())
}
