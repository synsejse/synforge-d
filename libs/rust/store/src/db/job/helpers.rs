use super::*;
use diesel_async::RunQueryDsl;

pub(in crate::db) async fn load_artifacts_map_for_rows<'a>(
    conn: &mut AsyncMysqlConnection,
    rows: impl IntoIterator<Item = &'a JobRecord>,
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    let job_ids = rows
        .into_iter()
        .map(|row| Uuid::parse_str(&row.id))
        .collect::<Result<Vec<_>, _>>()?;
    load_artifacts_map_for_job_ids(conn, &job_ids).await
}

async fn load_artifacts_map_for_job_ids(
    conn: &mut AsyncMysqlConnection,
    job_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let job_ids = job_ids.iter().map(Uuid::to_string).collect::<Vec<_>>();
    let rows = build_artifacts::table
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .filter(build_artifacts::job_id.eq_any(&job_ids))
        .select((
            ArtifactRecord::as_select(),
            artifact_signatures::status.nullable(),
            artifact_signatures::error_message.nullable(),
        ))
        .load(conn)
        .await?;
    let mut map: HashMap<Uuid, Vec<BuildArtifact>> = HashMap::new();
    for (row, signing_status, signing_error_message) in rows {
        let job_id = Uuid::parse_str(&row.job_id)?;
        map.entry(job_id).or_default().push(BuildArtifact {
            id: Uuid::parse_str(&row.id)?,
            package_name: row.package_name,
            mock_chroot: row.mock_chroot,
            file: PathBuf::from(row.file),
            sha256: row.sha256,
            size_bytes: row.size_bytes as u64,
            kind: row.kind,
            signing_status,
            signing_error_message,
        });
    }
    Ok(map)
}
