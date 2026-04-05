use super::*;

pub(in crate::db) async fn get_last_successful_revision(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
) -> anyhow::Result<Option<String>> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    store
        .with_connection(move |conn| {
            Ok(build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
                .filter(build_jobs::status.eq(BuildStatus::Succeeded))
                .order(build_jobs::finished_at.desc())
                .select(build_jobs::revision)
                .first::<String>(conn)
                .optional()?)
        })
        .await
}

pub(in crate::db) async fn has_active_job_for_target(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
) -> anyhow::Result<bool> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    store
        .with_connection(move |conn| {
            let active_job = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
                .filter(
                    build_jobs::status
                        .eq(BuildStatus::Pending)
                        .or(build_jobs::status.eq(BuildStatus::Running)),
                )
                .select(build_jobs::id)
                .first::<String>(conn)
                .optional()?;
            Ok(active_job.is_some())
        })
        .await
}

pub(in crate::db) async fn list_jobs(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    status: Option<BuildStatus>,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    completed_only: bool,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    store
        .with_connection(move |conn| {
            let mut query = build_jobs::table.into_boxed();
            if completed_only {
                query = query.filter(
                    build_jobs::status
                        .ne(BuildStatus::Pending)
                        .and(build_jobs::status.ne(BuildStatus::Running)),
                );
            }
            if let Some(status) = status {
                query = query.filter(build_jobs::status.eq(status));
            }
            if let Some(package_name) = package_name.as_deref() {
                let search = format!("%{}%", package_name);
                query = query.filter(build_jobs::package_name.like(search));
            }
            if let Some(mock_chroot) = mock_chroot.as_deref() {
                let search = format!("%{}%", mock_chroot);
                query = query.filter(build_jobs::mock_chroot.like(search));
            }
            let rows = query
                .order(build_jobs::created_at.desc())
                .limit(limit as i64)
                .offset(offset as i64)
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
}

pub(in crate::db) async fn count_jobs(
    store: &DieselStore,
    status: Option<BuildStatus>,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    completed_only: bool,
) -> anyhow::Result<u64> {
    store
        .with_connection(move |conn| {
            let mut query = build_jobs::table.into_boxed();
            if completed_only {
                query = query.filter(
                    build_jobs::status
                        .ne(BuildStatus::Pending)
                        .and(build_jobs::status.ne(BuildStatus::Running)),
                );
            }
            if let Some(status) = status {
                query = query.filter(build_jobs::status.eq(status));
            }
            if let Some(package_name) = package_name.as_deref() {
                let search = format!("%{}%", package_name);
                query = query.filter(build_jobs::package_name.like(search));
            }
            if let Some(mock_chroot) = mock_chroot.as_deref() {
                let search = format!("%{}%", mock_chroot);
                query = query.filter(build_jobs::mock_chroot.like(search));
            }
            let count = query.count().get_result::<i64>(conn)?;
            Ok(count as u64)
        })
        .await
}

pub(in crate::db) async fn list_active_jobs(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    package_name: Option<String>,
    mock_chroot: Option<String>,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    store
        .with_connection(move |conn| {
            let mut query = build_jobs::table.into_boxed().filter(
                build_jobs::status
                    .eq(BuildStatus::Pending)
                    .or(build_jobs::status.eq(BuildStatus::Running)),
            );
            if let Some(package_name) = package_name.as_deref() {
                let search = format!("%{}%", package_name);
                query = query.filter(build_jobs::package_name.like(search));
            }
            if let Some(mock_chroot) = mock_chroot.as_deref() {
                let search = format!("%{}%", mock_chroot);
                query = query.filter(build_jobs::mock_chroot.like(search));
            }
            let rows = query
                .order(build_jobs::created_at.desc())
                .limit(limit as i64)
                .offset(offset as i64)
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
}

pub(in crate::db) async fn count_active_jobs(
    store: &DieselStore,
    package_name: Option<String>,
    mock_chroot: Option<String>,
) -> anyhow::Result<u64> {
    store
        .with_connection(move |conn| {
            let mut query = build_jobs::table.into_boxed().filter(
                build_jobs::status
                    .eq(BuildStatus::Pending)
                    .or(build_jobs::status.eq(BuildStatus::Running)),
            );
            if let Some(package_name) = package_name.as_deref() {
                let search = format!("%{}%", package_name);
                query = query.filter(build_jobs::package_name.like(search));
            }
            if let Some(mock_chroot) = mock_chroot.as_deref() {
                let search = format!("%{}%", mock_chroot);
                query = query.filter(build_jobs::mock_chroot.like(search));
            }
            let count = query.count().get_result::<i64>(conn)?;
            Ok(count as u64)
        })
        .await
}

pub(in crate::db) async fn list_jobs_for_package(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let package_name = package_name.to_string();
    store
        .with_connection(move |conn| {
            let rows = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .order(build_jobs::created_at.desc())
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
}

pub(in crate::db) async fn get_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Option<BuildJobResponse>> {
    let job_id = job_id.to_string();
    store
        .with_connection(move |conn| {
            let row = build_jobs::table
                .find(job_id.as_str())
                .select(JobRecord::as_select())
                .first(conn)
                .optional()?;
            let artifacts = helpers::load_artifacts_map_for_rows(conn, row.as_ref().into_iter())?;
            row.map(|row| build_job_response_from_row(row, &artifacts))
                .transpose()
        })
        .await
}

pub(in crate::db) async fn list_prunable_successful_job_ids(
    store: &DieselStore,
    package_name: &str,
    mock_chroot: &str,
    keep: usize,
) -> anyhow::Result<Vec<Uuid>> {
    let package_name = package_name.to_string();
    let mock_chroot = mock_chroot.to_string();
    store
        .with_connection(move |conn| {
            let rows = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
                .filter(build_jobs::status.eq(BuildStatus::Succeeded))
                .order(build_jobs::finished_at.desc())
                .limit(i64::MAX)
                .offset(keep as i64)
                .select(build_jobs::id)
                .load::<String>(conn)?;
            rows.into_iter()
                .map(|id| Ok(Uuid::parse_str(&id)?))
                .collect()
        })
        .await
}

pub(in crate::db) async fn list_build_logs_for_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Vec<BuildLogRecord>> {
    let job_id = job_id.to_string();
    store
        .with_connection(move |conn| {
            Ok(build_logs::table
                .filter(build_logs::job_id.eq(job_id.as_str()))
                .order(build_logs::file.asc())
                .select((build_logs::file,))
                .load(conn)?)
        })
        .await
}

pub(in crate::db) async fn get_build_log_for_job_source(
    store: &DieselStore,
    job_id: Uuid,
    file: &str,
) -> anyhow::Result<Option<BuildLogRecord>> {
    let job_id = job_id.to_string();
    let file = file.to_string();
    store
        .with_connection(move |conn| {
            Ok(build_logs::table
                .find((job_id.as_str(), file.as_str()))
                .select((build_logs::file,))
                .first(conn)
                .optional()?)
        })
        .await
}
