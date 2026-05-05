use super::*;
use diesel_async::RunQueryDsl;

pub(in crate::db) async fn list_jobs_for_package(
    store: &DieselStore,
    package_name: &str,
    include_deleted: bool,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let package_name = package_name.to_string();
    let mut conn = store.get_connection().await?;
    let mut query = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name.as_str()))
        .into_boxed();
    if !include_deleted {
        query = query.filter(build_jobs::deleted_at.is_null());
    }
    let rows = query
        .order(build_jobs::created_at.desc())
        .select(JobRecord::as_select())
        .load(&mut conn)
        .await?;
    load_job_responses(&mut conn, rows).await
}

pub(in crate::db) async fn get_job(
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
    row.map(|row| build_job_response_from_row(row, &artifacts))
        .transpose()
}

pub(in crate::db) async fn filter_existing_job_ids(
    store: &DieselStore,
    candidate_ids: Vec<Uuid>,
) -> anyhow::Result<std::collections::HashSet<Uuid>> {
    use std::collections::HashSet;
    if candidate_ids.is_empty() {
        return Ok(HashSet::new());
    }
    let mut conn = store.get_connection().await?;
    // Postgres caps the bind-parameter count per statement at 65535;
    // chunk well below that to leave room for other parameters.
    const CHUNK_SIZE: usize = 1024;
    let mut existing: HashSet<Uuid> = HashSet::with_capacity(candidate_ids.len());
    for chunk in candidate_ids.chunks(CHUNK_SIZE) {
        let rows: Vec<Uuid> = build_jobs::table
            .filter(build_jobs::id.eq_any(chunk))
            .select(build_jobs::id)
            .load(&mut conn)
            .await?;
        existing.extend(rows);
    }
    Ok(existing)
}

pub(in crate::db) async fn list_build_logs_for_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Vec<BuildLogRecord>> {
    let mut conn = store.get_connection().await?;
    Ok(build_logs::table
        .filter(build_logs::job_id.eq(job_id))
        .order(build_logs::file.asc())
        .select((build_logs::file,))
        .load(&mut conn)
        .await?)
}

pub(in crate::db) async fn get_build_log_for_job_source(
    store: &DieselStore,
    job_id: Uuid,
    file: &str,
) -> anyhow::Result<Option<BuildLogRecord>> {
    let file = file.to_string();
    let mut conn = store.get_connection().await?;
    Ok(build_logs::table
        .find((job_id, file.as_str()))
        .select((build_logs::file,))
        .first(&mut conn)
        .await
        .optional()?)
}
