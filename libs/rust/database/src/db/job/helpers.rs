use super::*;
use diesel_async::RunQueryDsl;

pub(in crate::db) async fn load_artifacts_map_for_rows<'a>(
    conn: &mut AsyncPgConnection,
    rows: impl IntoIterator<Item = &'a JobRecord>,
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    let job_ids = rows.into_iter().map(|row| row.id).collect::<Vec<_>>();
    load_artifacts_map_for_job_ids(conn, &job_ids).await
}

pub(in crate::db) async fn load_ccache_stats_map_for_rows<'a>(
    conn: &mut AsyncPgConnection,
    rows: impl IntoIterator<Item = &'a JobRecord>,
) -> anyhow::Result<HashMap<Uuid, BuildCcacheStats>> {
    let job_ids = rows.into_iter().map(|row| row.id).collect::<Vec<_>>();
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let records = build_ccache_stats::table
        .filter(build_ccache_stats::job_id.eq_any(job_ids))
        .select(BuildCcacheStatsRecord::as_select())
        .load(conn)
        .await?;
    records
        .into_iter()
        .map(|record| {
            let job_id = record.job_id;
            Ok((job_id, build_ccache_stats_from_record(record)?))
        })
        .collect()
}

fn build_ccache_stats_from_record(
    record: BuildCcacheStatsRecord,
) -> anyhow::Result<BuildCcacheStats> {
    Ok(BuildCcacheStats {
        compiler_calls: nonnegative_u64(record.compiler_calls, "compiler_calls")?,
        direct_hits: nonnegative_u64(record.direct_hits, "direct_hits")?,
        preprocessed_hits: nonnegative_u64(record.preprocessed_hits, "preprocessed_hits")?,
        cache_misses: nonnegative_u64(record.cache_misses, "cache_misses")?,
        uncacheable_calls: nonnegative_u64(record.uncacheable_calls, "uncacheable_calls")?,
        error_calls: nonnegative_u64(record.error_calls, "error_calls")?,
    })
}

fn nonnegative_u64(value: i64, field: &str) -> anyhow::Result<u64> {
    u64::try_from(value)
        .map_err(|_| anyhow::anyhow!("build ccache statistic {field} is negative: {value}"))
}

async fn load_artifacts_map_for_job_ids(
    conn: &mut AsyncPgConnection,
    job_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = build_artifacts::table
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .filter(build_artifacts::job_id.eq_any(job_ids))
        .select((
            ArtifactRecord::as_select(),
            artifact_signatures::status.nullable(),
            artifact_signatures::error_message.nullable(),
        ))
        .load(conn)
        .await?;
    let mut map: HashMap<Uuid, Vec<BuildArtifact>> = HashMap::new();
    for (row, signing_status, signing_error_message) in rows {
        let job_id = row.job_id;
        map.entry(job_id).or_default().push(BuildArtifact {
            id: row.id,
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
