use diesel::pg::Pg;
use diesel_async::RunQueryDsl;

use super::*;

fn apply_status_filters<'a>(
    mut query: build_jobs::BoxedQuery<'a, Pg>,
    status: Option<BuildStatus>,
    completed_only: bool,
) -> build_jobs::BoxedQuery<'a, Pg> {
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
    query
}

fn apply_search_filters<'a>(
    mut query: build_jobs::BoxedQuery<'a, Pg>,
    package_name: Option<&str>,
    mock_chroot: Option<&str>,
) -> build_jobs::BoxedQuery<'a, Pg> {
    if let Some(package_name) = package_name {
        query = query.filter(build_jobs::package_name.like(format!("%{}%", package_name)));
    }
    if let Some(mock_chroot) = mock_chroot {
        query = query.filter(build_jobs::mock_chroot.like(format!("%{}%", mock_chroot)));
    }
    query
}

fn apply_deleted_filter<'a>(
    query: build_jobs::BoxedQuery<'a, Pg>,
    include_deleted: bool,
) -> build_jobs::BoxedQuery<'a, Pg> {
    if include_deleted {
        query
    } else {
        query.filter(build_jobs::deleted_at.is_null())
    }
}

fn apply_active_job_filter<'a>(
    query: build_jobs::BoxedQuery<'a, Pg>,
) -> build_jobs::BoxedQuery<'a, Pg> {
    query.filter(
        build_jobs::status
            .eq(BuildStatus::Pending)
            .or(build_jobs::status.eq(BuildStatus::Running)),
    )
}

pub(in crate::db) async fn list_jobs(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    status: Option<BuildStatus>,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    completed_only: bool,
    include_deleted: bool,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let mut conn = store.get_connection().await?;
    let query = apply_deleted_filter(
        apply_search_filters(
            apply_status_filters(build_jobs::table.into_boxed(), status, completed_only),
            package_name.as_deref(),
            mock_chroot.as_deref(),
        ),
        include_deleted,
    );
    let rows = query
        .order(build_jobs::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(JobRecord::as_select())
        .load(&mut conn)
        .await?;
    load_job_responses(&mut conn, rows).await
}

pub(in crate::db) async fn count_jobs(
    store: &DieselStore,
    status: Option<BuildStatus>,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    completed_only: bool,
    include_deleted: bool,
) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let count = apply_deleted_filter(
        apply_search_filters(
            apply_status_filters(build_jobs::table.into_boxed(), status, completed_only),
            package_name.as_deref(),
            mock_chroot.as_deref(),
        ),
        include_deleted,
    )
    .count()
    .get_result::<i64>(&mut conn)
    .await?;
    Ok(count as u64)
}

pub(in crate::db) async fn list_active_jobs(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    package_name: Option<String>,
    mock_chroot: Option<String>,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let mut conn = store.get_connection().await?;
    let query = apply_search_filters(
        apply_active_job_filter(build_jobs::table.into_boxed()),
        package_name.as_deref(),
        mock_chroot.as_deref(),
    );
    let rows = query
        .order(build_jobs::created_at.desc())
        .limit(limit as i64)
        .offset(offset as i64)
        .select(JobRecord::as_select())
        .load(&mut conn)
        .await?;
    load_job_responses(&mut conn, rows).await
}

pub(in crate::db) async fn count_active_jobs(
    store: &DieselStore,
    package_name: Option<String>,
    mock_chroot: Option<String>,
) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let count = apply_search_filters(
        apply_active_job_filter(build_jobs::table.into_boxed()),
        package_name.as_deref(),
        mock_chroot.as_deref(),
    )
    .count()
    .get_result::<i64>(&mut conn)
    .await?;
    Ok(count as u64)
}
