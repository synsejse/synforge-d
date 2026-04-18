use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};

use super::super::*;

pub(in crate::db) async fn insert_job(store: &DieselStore, job: &BuildJob) -> anyhow::Result<()> {
    let job = job.clone();
    let mut conn = store.get_connection().await?;
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
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub(in crate::db) async fn set_job_running(
    store: &DieselStore,
    job_id: Uuid,
    worker_container_id: Option<&str>,
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let now = format_timestamp(now_utc());
    let worker_container_id = worker_container_id.map(ToOwned::to_owned);
    let mut conn = store.get_connection().await?;
    diesel::update(build_jobs::table.find(job_id.as_str()))
        .set((
            build_jobs::status.eq(BuildStatus::Running),
            build_jobs::updated_at.eq(now.as_str()),
            build_jobs::worker_container_id.eq(worker_container_id.as_deref()),
        ))
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub(in crate::db) async fn reset_job_for_retry(
    store: &DieselStore,
    job_id: Uuid,
    trigger: BuildTrigger,
    revision: &str,
) -> anyhow::Result<()> {
    let job_id = job_id.to_string();
    let revision = revision.to_string();
    let now = format_timestamp(now_utc());
    let mut conn = store.get_connection().await?;
    conn.transaction::<(), anyhow::Error, _>(|conn| {
        async move {
            diesel::delete(
                published_repo_files::table.filter(
                    published_repo_files::artifact_id.eq_any(
                        build_artifacts::table
                            .filter(build_artifacts::job_id.eq(job_id.as_str()))
                            .select(build_artifacts::id),
                    ),
                ),
            )
            .execute(conn)
            .await?;

            diesel::delete(
                build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())),
            )
            .execute(conn)
            .await?;

            diesel::delete(build_logs::table.filter(build_logs::job_id.eq(job_id.as_str())))
                .execute(conn)
                .await?;

            diesel::update(build_jobs::table.find(job_id.as_str()))
                .set((
                    build_jobs::trigger.eq(trigger),
                    build_jobs::revision.eq(revision.as_str()),
                    build_jobs::status.eq(BuildStatus::Pending),
                    build_jobs::worker_container_id.eq::<Option<&str>>(None),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::finished_at.eq::<Option<&str>>(None),
                    build_jobs::error_message.eq::<Option<&str>>(None),
                ))
                .execute(conn)
                .await?;
            Ok(())
        }
        .scope_boxed()
    })
    .await?;
    Ok(())
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
    let mut conn = store.get_connection().await?;
    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        async move {
            let now = format_timestamp(now_utc());
            let job_row = build_jobs::table
                .find(job_id.as_str())
                .select(JobRecord::as_select())
                .first(conn)
                .await?;

            diesel::update(build_jobs::table.find(job_id.as_str()))
                .set((
                    build_jobs::status.eq(status),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::finished_at.eq(Some(now.as_str())),
                    build_jobs::error_message.eq(error_message.as_deref()),
                ))
                .execute(conn)
                .await?;

            diesel::delete(
                build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())),
            )
            .execute(conn)
            .await?;

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
                    .execute(conn)
                    .await?;
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
                    .execute(conn)
                    .await?;
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
            .execute(conn)
            .await?;

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
