use super::*;

pub(super) async fn get_last_successful_revision(
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
                .filter(build_jobs::status.eq(BuildStatus::Succeeded.as_db_text()))
                .order(build_jobs::finished_at.desc())
                .select(build_jobs::revision)
                .first::<String>(conn)
                .optional()?)
        })
        .await
}

pub(super) async fn has_active_job(store: &DieselStore, package_name: &str) -> anyhow::Result<bool> {
    let package_name = package_name.to_string();
    store
        .with_connection(move |conn| {
            let active_job = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(
                    build_jobs::status
                        .eq(BuildStatus::Pending.as_db_text())
                        .or(build_jobs::status.eq(BuildStatus::Running.as_db_text())),
                )
                .select(build_jobs::id)
                .first::<String>(conn)
                .optional()?;
            Ok(active_job.is_some())
        })
        .await
}

pub(super) async fn insert_job(store: &DieselStore, job: &BuildJob) -> anyhow::Result<()> {
    let job = job.clone();
    store
        .with_connection(move |conn| {
            let id = job.id.to_string();
            let spec_path = job.spec_path.to_string_lossy().to_string();
            let new_job = NewJobRecord {
                id: id.as_str(),
                package_name: job.package_name.as_str(),
                mock_chroot: job.mock_chroot.as_str(),
                revision: job.revision.as_str(),
                trigger: job.trigger.as_db_text(),
                status: job.status.as_db_text(),
                spec_path: spec_path.as_str(),
                worker_container_id: job.worker_container_id.as_deref(),
                created_at: format_timestamp(job.created_at),
                updated_at: format_timestamp(job.updated_at),
                finished_at: job.finished_at.map(format_timestamp),
                error_message: job.error_message.as_deref(),
            };
            diesel::insert_into(build_jobs::table)
                .values(&new_job)
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(super) async fn set_job_running(
    store: &DieselStore,
    job_id: Uuid,
    worker_container_id: Option<&str>,
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let now = format_timestamp(now_utc());
    let worker_container_id = worker_container_id.map(ToOwned::to_owned);
    store
        .with_connection(move |conn| {
            diesel::update(build_jobs::table.find(job_id.as_str()))
                .set((
                    build_jobs::status.eq(BuildStatus::Running.as_db_text()),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::worker_container_id.eq(worker_container_id.as_deref()),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(super) async fn finish_job(
    store: &DieselStore,
    job_id: Uuid,
    status: BuildStatus,
    error_message: Option<&str>,
    artifacts: &[BuildArtifact],
    published_files: &[PublishedRepoFile],
    logs_path: Option<&Path>,
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let status_value = status.as_db_text().to_string();
    let error_message = error_message.map(ToOwned::to_owned);
    let logs_path = logs_path.map(Path::to_path_buf);
    let artifacts = artifacts.to_vec();
    let published_files = published_files.to_vec();
    store
        .with_connection(move |conn| {
            conn.transaction::<(), diesel::result::Error, _>(|conn| {
                let now = format_timestamp(now_utc());
                let job_row = build_jobs::table
                    .find(job_id.as_str())
                    .select(JobRecord::as_select())
                    .first(conn)?;

                diesel::update(build_jobs::table.find(job_id.as_str()))
                    .set((
                        build_jobs::status.eq(status_value.as_str()),
                        build_jobs::updated_at.eq(now.as_str()),
                        build_jobs::finished_at.eq(Some(now.as_str())),
                        build_jobs::error_message.eq(error_message.as_deref()),
                    ))
                    .execute(conn)?;

                diesel::delete(build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())))
                    .execute(conn)?;

                if !artifacts.is_empty() {
                    let rows = artifacts
                        .iter()
                        .map(|artifact| NewArtifactRecord {
                            job_id: job_id.clone(),
                            package_name: job_row.package_name.clone(),
                            mock_chroot: job_row.mock_chroot.clone(),
                            arch: artifact.arch.clone(),
                            path: artifact.path.to_string_lossy().to_string(),
                            relative_repo_path: artifact.relative_repo_path.to_string_lossy().to_string(),
                            sha256: artifact.sha256.clone(),
                            size_bytes: artifact.size_bytes as i64,
                            kind: artifact.kind.as_db_text().to_string(),
                        })
                        .collect::<Vec<_>>();
                    diesel::insert_into(build_artifacts::table)
                        .values(&rows)
                        .execute(conn)?;
                }

                if let Some(logs_path) = &logs_path {
                    let log_path = logs_path.to_string_lossy().to_string();
                    let log_row = NewBuildLogRecord {
                        job_id: job_id.as_str(),
                        log_path: log_path.as_str(),
                        updated_at: now.as_str(),
                    };
                    diesel::insert_into(build_logs::table)
                        .values(&log_row)
                        .on_conflict(build_logs::job_id)
                        .do_update()
                        .set((
                            build_logs::log_path.eq(log_path.as_str()),
                            build_logs::updated_at.eq(now.as_str()),
                        ))
                        .execute(conn)?;
                }

                diesel::delete(
                    published_repo_files::table.filter(published_repo_files::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;

                if !published_files.is_empty() {
                    let rows = published_files
                        .iter()
                        .map(|file| NewPublishedRepoFileRecord {
                            job_id: job_id.clone(),
                            package_name: file.package_name.clone(),
                            mock_chroot: file.mock_chroot.clone(),
                            arch: file.arch.clone(),
                            repo_path: file.repo_path.to_string_lossy().to_string(),
                            sha256: file.sha256.clone(),
                            size_bytes: file.size_bytes as i64,
                            kind: file.kind.as_db_text().to_string(),
                            published_at: format_timestamp(file.published_at),
                        })
                        .collect::<Vec<_>>();
                    diesel::insert_into(published_repo_files::table)
                        .values(&rows)
                        .execute(conn)?;
                }

                Ok(())
            })?;
            Ok(())
        })
        .await
}

pub(super) async fn list_jobs(
    store: &DieselStore,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    store
        .with_connection(move |conn| {
            let rows = build_jobs::table
                .order(build_jobs::created_at.desc())
                .limit(limit as i64)
                .offset(offset as i64)
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
}

pub(super) async fn list_jobs_for_package(
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

pub(super) async fn get_job(
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
            let artifacts = load_artifacts_map_for_rows(conn, row.as_ref().into_iter())?;
            row.map(|row| build_job_response_from_row(row, &artifacts))
                .transpose()
        })
        .await
}

pub(super) async fn delete_job(
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
            let artifacts = load_artifacts_map_for_rows(conn, row.as_ref().into_iter())?;
            let Some(row) = row else {
                return Ok(None);
            };
            let response = build_job_response_from_row(row, &artifacts)?;
            if matches!(response.job.status, BuildStatus::Pending | BuildStatus::Running) {
                return Err(anyhow::anyhow!("cannot delete a pending or running job"));
            }

            conn.transaction::<(), anyhow::Error, _>(|conn| {
                diesel::delete(build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())))
                    .execute(conn)?;
                diesel::delete(build_logs::table.filter(build_logs::job_id.eq(job_id.as_str())))
                    .execute(conn)?;
                diesel::delete(
                    published_repo_files::table.filter(published_repo_files::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;
                diesel::delete(build_jobs::table.find(job_id.as_str())).execute(conn)?;
                Ok(())
            })?;

            Ok(Some(response))
        })
        .await
}

pub(super) async fn abort_unfinished_jobs(
    store: &DieselStore,
    message: &str,
) -> anyhow::Result<()> {
    let message = message.to_string();
    store
        .with_connection(move |conn| {
            let now = format_timestamp(now_utc());
            diesel::update(build_jobs::table.filter(build_jobs::finished_at.is_null()))
                .set((
                    build_jobs::status.eq(BuildStatus::Failed.as_db_text()),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::finished_at.eq(Some(now.as_str())),
                    build_jobs::error_message.eq(Some(message.as_str())),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(super) async fn list_prunable_successful_job_ids(
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
                .filter(build_jobs::status.eq(BuildStatus::Succeeded.as_db_text()))
                .order(build_jobs::finished_at.desc())
                .offset(keep as i64)
                .select(build_jobs::id)
                .load::<String>(conn)?;
            rows.into_iter().map(|id| Ok(Uuid::parse_str(&id)?)).collect()
        })
        .await
}

pub(super) fn load_artifacts_map_for_rows<'a>(
    conn: &mut SqliteConnection,
    rows: impl IntoIterator<Item = &'a JobRecord>,
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    let job_ids = rows
        .into_iter()
        .map(|row| Uuid::parse_str(&row.id))
        .collect::<Result<Vec<_>, _>>()?;
    load_artifacts_map_for_job_ids(conn, &job_ids)
}

fn load_artifacts_map_for_job_ids(
    conn: &mut SqliteConnection,
    job_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let job_ids = job_ids.iter().map(Uuid::to_string).collect::<Vec<_>>();
    let rows = build_artifacts::table
        .filter(build_artifacts::job_id.eq_any(&job_ids))
        .select(ArtifactRecord::as_select())
        .load(conn)?;
    let mut map: HashMap<Uuid, Vec<BuildArtifact>> = HashMap::new();
    for row in rows {
        let job_id = Uuid::parse_str(&row.job_id)?;
        map.entry(job_id).or_default().push(BuildArtifact {
            package_name: row.package_name,
            mock_chroot: row.mock_chroot,
            arch: row.arch,
            path: PathBuf::from(row.path),
            relative_repo_path: PathBuf::from(row.relative_repo_path),
            sha256: row.sha256,
            size_bytes: row.size_bytes as u64,
            kind: ArtifactKind::from_db_text(&row.kind),
        });
    }
    Ok(map)
}
