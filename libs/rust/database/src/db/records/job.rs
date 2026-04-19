use super::super::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_jobs)]
pub(crate) struct JobRecord {
    pub(crate) id: Uuid,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) revision: String,
    pub(crate) trigger: BuildTrigger,
    pub(crate) status: BuildStatus,
    pub(crate) spec_file: String,
    pub(crate) worker_container_id: Option<String>,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
    pub(crate) finished_at: Option<OffsetDateTime>,
    pub(crate) error_message: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = build_jobs)]
pub(crate) struct NewJobRecord<'a> {
    pub(crate) id: Uuid,
    pub(crate) package_name: &'a str,
    pub(crate) mock_chroot: &'a str,
    pub(crate) revision: &'a str,
    pub(crate) trigger: BuildTrigger,
    pub(crate) status: BuildStatus,
    pub(crate) spec_file: &'a str,
    pub(crate) worker_container_id: Option<&'a str>,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
    pub(crate) finished_at: Option<OffsetDateTime>,
    pub(crate) error_message: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_artifacts)]
pub(crate) struct ArtifactRecord {
    pub(crate) id: Uuid,
    pub(crate) job_id: Uuid,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) file: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
}

#[derive(Insertable)]
#[diesel(table_name = build_artifacts)]
pub(crate) struct NewArtifactRecord {
    pub(crate) id: Uuid,
    pub(crate) job_id: Uuid,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) file: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
}

#[derive(Insertable)]
#[diesel(table_name = artifact_signatures)]
pub(crate) struct NewArtifactSignatureRecord {
    pub(crate) artifact_id: Uuid,
    pub(crate) status: ArtifactSigningStatus,
    pub(crate) signed_at: Option<OffsetDateTime>,
    pub(crate) key_id: Option<String>,
    pub(crate) fingerprint: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = build_logs)]
pub(crate) struct NewBuildLogRecord<'a> {
    pub(crate) job_id: Uuid,
    pub(crate) file: &'a str,
    pub(crate) updated_at: OffsetDateTime,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_failure_backoff)]
pub(crate) struct BuildFailureBackoffRecord {
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) consecutive_failures: i32,
    pub(crate) next_eligible_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = build_failure_backoff)]
pub(crate) struct NewBuildFailureBackoffRecord<'a> {
    pub(crate) package_name: &'a str,
    pub(crate) mock_chroot: &'a str,
    pub(crate) consecutive_failures: i32,
    pub(crate) next_eligible_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Debug, Queryable)]
pub struct BuildLogRecord {
    pub file: String,
}

pub(crate) async fn load_job_responses(
    conn: &mut AsyncPgConnection,
    rows: Vec<JobRecord>,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let artifacts = job::load_artifacts_map_for_rows(conn, rows.iter()).await?;
    rows.into_iter()
        .map(|row| build_job_response_from_row(row, &artifacts))
        .collect()
}

pub(crate) fn build_job_response_from_row(
    row: JobRecord,
    artifacts: &HashMap<Uuid, Vec<BuildArtifact>>,
) -> anyhow::Result<BuildJobResponse> {
    let job = job_from_row(row)?;
    Ok(BuildJobResponse {
        artifacts: artifacts.get(&job.id).cloned().unwrap_or_default(),
        job,
    })
}

pub(crate) fn job_from_row(row: JobRecord) -> anyhow::Result<BuildJob> {
    Ok(BuildJob {
        id: row.id,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        revision: row.revision,
        trigger: row.trigger,
        status: row.status,
        spec_file: PathBuf::from(row.spec_file),
        worker_container_id: row.worker_container_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        finished_at: row.finished_at,
        error_message: row.error_message,
    })
}
