use super::*;
use crate::db::traits::BuildFailureBackoffState;
use diesel_async::RunQueryDsl;

pub(in crate::db) async fn get_last_successful_revision(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
) -> anyhow::Result<Option<String>> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    let mut conn = store.get_connection().await?;
    Ok(build_jobs::table
        .filter(build_jobs::package_name.eq(package_name.as_str()))
        .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .order(build_jobs::finished_at.desc())
        .select(build_jobs::revision)
        .first::<String>(&mut conn)
        .await
        .optional()?)
}

pub(in crate::db) async fn has_active_job_for_target(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
) -> anyhow::Result<bool> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    let mut conn = store.get_connection().await?;
    let active_job = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name.as_str()))
        .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Pending)
                .or(build_jobs::status.eq(BuildStatus::Running)),
        )
        .select(build_jobs::id)
        .first::<Uuid>(&mut conn)
        .await
        .optional()?;
    Ok(active_job.is_some())
}

pub(in crate::db) async fn list_prunable_successful_job_ids(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
    keep: usize,
) -> anyhow::Result<Vec<Uuid>> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    let mut conn = store.get_connection().await?;
    let rows = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name.as_str()))
        .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .order(build_jobs::finished_at.desc())
        .limit(i64::MAX)
        .offset(keep as i64)
        .select(build_jobs::id)
        .load::<Uuid>(&mut conn)
        .await?;
    Ok(rows)
}

pub(in crate::db) async fn get_target_build_backoff(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
) -> anyhow::Result<Option<BuildFailureBackoffState>> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    let mut conn = store.get_connection().await?;
    let record = build_failure_backoff::table
        .find((package_name.as_str(), mock_chroot.as_str()))
        .select(BuildFailureBackoffRecord::as_select())
        .first(&mut conn)
        .await
        .optional()?;
    record
        .map(|record| {
            Ok(BuildFailureBackoffState {
                consecutive_failures: record.consecutive_failures as u32,
                next_eligible_at: record.next_eligible_at,
            })
        })
        .transpose()
}

pub(in crate::db) async fn list_target_build_backoffs(
    store: &DieselStore,
) -> anyhow::Result<Vec<(String, String, BuildFailureBackoffState)>> {
    let mut conn = store.get_connection().await?;
    let rows = build_failure_backoff::table
        .select(BuildFailureBackoffRecord::as_select())
        .load::<BuildFailureBackoffRecord>(&mut conn)
        .await?;
    Ok(rows
        .into_iter()
        .map(|record| {
            (
                record.package_name,
                record.mock_chroot,
                BuildFailureBackoffState {
                    consecutive_failures: record.consecutive_failures as u32,
                    next_eligible_at: record.next_eligible_at,
                },
            )
        })
        .collect())
}
