use super::*;

pub(in crate::db) async fn insert_job(store: &DieselStore, job: &BuildJob) -> anyhow::Result<()> {
    let job = job.clone();
    store
        .with_connection(move |conn| {
            let id = job.id.to_string();
            let spec_file = job.spec_file.to_string_lossy().to_string();
            let new_job = NewJobRecord {
                id: id.as_str(),
                package_name: job.package_name.as_str(),
                mock_chroot: job.mock_chroot.as_str(),
                revision: job.revision.as_str(),
                trigger: job.trigger,
                status: job.status,
                spec_file: spec_file.as_str(),
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

pub(in crate::db) async fn set_job_running(
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
                    build_jobs::status.eq(BuildStatus::Running),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::worker_container_id.eq(worker_container_id.as_deref()),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(in crate::db) async fn finish_job(
    store: &DieselStore,
    job_id: Uuid,
    status: BuildStatus,
    error_message: Option<&str>,
    artifacts: &[BuildArtifact],
    published_files: &[PublishedRepoFile],
    artifact_signatures: &[ArtifactSignature],
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let error_message = error_message.map(ToOwned::to_owned);
    let artifacts = artifacts.to_vec();
    let published_files = published_files.to_vec();
    let artifact_signatures = artifact_signatures.to_vec();
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
                        build_jobs::status.eq(status),
                        build_jobs::updated_at.eq(now.as_str()),
                        build_jobs::finished_at.eq(Some(now.as_str())),
                        build_jobs::error_message.eq(error_message.as_deref()),
                    ))
                    .execute(conn)?;

                diesel::delete(
                    build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;

                if !artifacts.is_empty() {
                    let rows = artifacts
                        .iter()
                        .map(|artifact| NewArtifactRecord {
                            id: artifact.id.to_string(),
                            job_id: job_id.clone(),
                            package_name: job_row.package_name.clone(),
                            mock_chroot: job_row.mock_chroot.clone(),
                            file: artifact.file.to_string_lossy().to_string(),
                            sha256: artifact.sha256.clone(),
                            size_bytes: artifact.size_bytes as i64,
                            kind: artifact.kind,
                        })
                        .collect::<Vec<_>>();
                    diesel::insert_into(build_artifacts::table)
                        .values(&rows)
                        .execute(conn)?;
                }

                if !artifact_signatures.is_empty() {
                    let rows = artifact_signatures
                        .iter()
                        .map(|signature| NewArtifactSignatureRecord {
                            artifact_id: signature.artifact_id.to_string(),
                            status: signature.status,
                            signed_at: signature.signed_at.map(format_timestamp),
                            key_id: signature.key_id.clone(),
                            fingerprint: signature.fingerprint.clone(),
                            error_message: signature.error_message.clone(),
                            updated_at: now.clone(),
                        })
                        .collect::<Vec<_>>();
                    diesel::insert_into(artifact_signatures::table)
                        .values(&rows)
                        .execute(conn)?;
                }

                diesel::delete(
                    published_repo_files::table.filter(
                        published_repo_files::artifact_id.eq_any(
                            build_artifacts::table
                                .filter(build_artifacts::job_id.eq(job_id.as_str()))
                                .select(build_artifacts::id),
                        ),
                    ),
                )
                .execute(conn)?;

                if !published_files.is_empty() {
                    let rows = published_files
                        .iter()
                        .map(|file| NewPublishedRepoFileRecord {
                            artifact_id: file.artifact_id.to_string(),
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

pub(in crate::db) async fn delete_job(
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
                diesel::delete(
                    build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;
                diesel::delete(build_logs::table.filter(build_logs::job_id.eq(job_id.as_str())))
                    .execute(conn)?;
                diesel::delete(
                    published_repo_files::table.filter(
                        published_repo_files::artifact_id.eq_any(
                            build_artifacts::table
                                .filter(build_artifacts::job_id.eq(job_id.as_str()))
                                .select(build_artifacts::id),
                        ),
                    ),
                )
                .execute(conn)?;
                diesel::delete(build_jobs::table.find(job_id.as_str())).execute(conn)?;
                Ok(())
            })?;

            Ok(Some(response))
        })
        .await
}

pub(in crate::db) async fn abort_unfinished_jobs(
    store: &DieselStore,
    message: &str,
) -> anyhow::Result<()> {
    let message = message.to_string();
    store
        .with_connection(move |conn| {
            let now = format_timestamp(now_utc());
            diesel::update(build_jobs::table.filter(build_jobs::finished_at.is_null()))
                .set((
                    build_jobs::status.eq(BuildStatus::Failed),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::finished_at.eq(Some(now.as_str())),
                    build_jobs::error_message.eq(Some(message.as_str())),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
}

pub(in crate::db) async fn upsert_build_log(
    store: &DieselStore,
    job_id: Uuid,
    file: &str,
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let file = file.to_string();
    let updated_at = format_timestamp(now_utc());
    store
        .with_connection(move |conn| {
            let existing: Option<String> = build_logs::table
                .find((job_id.as_str(), file.as_str()))
                .select(build_logs::file)
                .first(conn)
                .optional()?;

            if existing.is_some() {
                diesel::update(build_logs::table.find((job_id.as_str(), file.as_str())))
                    .set(build_logs::updated_at.eq(updated_at.as_str()))
                    .execute(conn)?;
            } else {
                let row = NewBuildLogRecord {
                    job_id: job_id.as_str(),
                    file: file.as_str(),
                    updated_at: updated_at.as_str(),
                };
                diesel::insert_into(build_logs::table)
                    .values(&row)
                    .execute(conn)?;
            }
            Ok(())
        })
        .await
}
