use super::*;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{
    dsl::{count, sum},
    expression_methods::AggregateExpressionMethods,
};
use diesel_async::RunQueryDsl;

type PublishedRepoRow = (
    Uuid,
    Uuid,
    String,
    String,
    String,
    String,
    i64,
    ArtifactKind,
    OffsetDateTime,
    Option<ArtifactSigningStatus>,
    Option<String>,
);

pub(super) async fn list_published_repo_files(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    kind: Option<ArtifactKind>,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let mut conn = store.get_connection().await?;
    let mut query = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .into_boxed();
    if let Some(package_name) = package_name.as_deref() {
        query = query.filter(build_artifacts::package_name.eq(package_name));
    }
    if let Some(mock_chroot) = mock_chroot.as_deref() {
        query = query.filter(build_artifacts::mock_chroot.eq(mock_chroot));
    }
    if let Some(kind) = kind {
        query = query.filter(build_artifacts::kind.eq(kind));
    }
    let rows = query
        .order((
            build_artifacts::package_name.asc(),
            published_repo_files::published_at.desc(),
            build_artifacts::file.asc(),
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .select((
            published_repo_files::artifact_id,
            build_artifacts::job_id,
            build_artifacts::package_name,
            build_artifacts::mock_chroot,
            build_artifacts::file,
            build_artifacts::sha256,
            build_artifacts::size_bytes,
            build_artifacts::kind,
            published_repo_files::published_at,
            artifact_signatures::status.nullable(),
            artifact_signatures::error_message.nullable(),
        ))
        .load::<PublishedRepoRow>(&mut conn)
        .await?;
    rows.into_iter()
        .map(published_repo_file_from_record)
        .collect()
}

pub(super) async fn list_published_repo_files_for_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let mut conn = store.get_connection().await?;
    load_published_repo_files_for_job(&mut conn, job_id).await
}

pub(super) async fn list_published_repo_files_for_package(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let package_name = package_name.to_string();
    let mut conn = store.get_connection().await?;
    let rows = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .filter(build_artifacts::package_name.eq(package_name.as_str()))
        .order((
            published_repo_files::published_at.desc(),
            build_artifacts::file.asc(),
        ))
        .select((
            published_repo_files::artifact_id,
            build_artifacts::job_id,
            build_artifacts::package_name,
            build_artifacts::mock_chroot,
            build_artifacts::file,
            build_artifacts::sha256,
            build_artifacts::size_bytes,
            build_artifacts::kind,
            published_repo_files::published_at,
            artifact_signatures::status.nullable(),
            artifact_signatures::error_message.nullable(),
        ))
        .load::<PublishedRepoRow>(&mut conn)
        .await?;
    rows.into_iter()
        .map(published_repo_file_from_record)
        .collect()
}

pub(super) async fn count_published_repo_files(
    store: &DieselStore,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    kind: Option<ArtifactKind>,
) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let mut query = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .into_boxed();
    if let Some(package_name) = package_name.as_deref() {
        query = query.filter(build_artifacts::package_name.eq(package_name));
    }
    if let Some(mock_chroot) = mock_chroot.as_deref() {
        query = query.filter(build_artifacts::mock_chroot.eq(mock_chroot));
    }
    if let Some(kind) = kind {
        query = query.filter(build_artifacts::kind.eq(kind));
    }
    let count = query.count().get_result::<i64>(&mut conn).await?;
    Ok(count as u64)
}

pub(super) async fn list_recent_published_repo_files(
    store: &DieselStore,
    limit: usize,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let mut conn = store.get_connection().await?;
    let rows = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .order((
            published_repo_files::published_at.desc(),
            build_artifacts::file.asc(),
        ))
        .limit(limit as i64)
        .select((
            published_repo_files::artifact_id,
            build_artifacts::job_id,
            build_artifacts::package_name,
            build_artifacts::mock_chroot,
            build_artifacts::file,
            build_artifacts::sha256,
            build_artifacts::size_bytes,
            build_artifacts::kind,
            published_repo_files::published_at,
            artifact_signatures::status.nullable(),
            artifact_signatures::error_message.nullable(),
        ))
        .load::<PublishedRepoRow>(&mut conn)
        .await?;
    rows.into_iter()
        .map(published_repo_file_from_record)
        .collect()
}

pub(super) async fn list_repo_target_summaries(
    store: &DieselStore,
) -> anyhow::Result<Vec<RepoTargetSummary>> {
    let mut conn = store.get_connection().await?;
    let rows = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .group_by(build_artifacts::mock_chroot)
        .select((
            build_artifacts::mock_chroot,
            count(build_artifacts::package_name).aggregate_distinct(),
            count(build_artifacts::job_id).aggregate_distinct(),
            sum(build_artifacts::size_bytes),
        ))
        .order(build_artifacts::mock_chroot.desc())
        .load::<(String, i64, i64, Option<BigDecimal>)>(&mut conn)
        .await?;

    let targets = rows
        .into_iter()
        .map(|(mock_chroot, package_count, build_count, size_bytes)| {
            Ok(RepoTargetSummary {
                mock_chroot,
                package_count: package_count as u64,
                build_count: build_count as u64,
                size_bytes: size_bytes
                    .map(decimal_to_u64)
                    .transpose()?
                    .unwrap_or_default(),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(targets)
}

pub(super) async fn get_repo_distinct_counts(
    store: &DieselStore,
) -> anyhow::Result<(u64, u64, u64)> {
    let mut conn = store.get_connection().await?;
    let package_count = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .select(count(build_artifacts::package_name).aggregate_distinct())
        .first::<i64>(&mut conn)
        .await? as u64;
    let target_count = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .select(count(build_artifacts::mock_chroot).aggregate_distinct())
        .first::<i64>(&mut conn)
        .await? as u64;
    let build_count = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .select(count(build_artifacts::job_id).aggregate_distinct())
        .first::<i64>(&mut conn)
        .await? as u64;

    Ok((package_count, target_count, build_count))
}

pub(super) async fn sum_published_repo_file_bytes(store: &DieselStore) -> anyhow::Result<u64> {
    let mut conn = store.get_connection().await?;
    let total_size = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .select(sum(build_artifacts::size_bytes))
        .first::<Option<BigDecimal>>(&mut conn)
        .await?;
    Ok(total_size
        .map(decimal_to_u64)
        .transpose()?
        .unwrap_or_default())
}

fn decimal_to_u64(value: BigDecimal) -> anyhow::Result<u64> {
    value
        .to_u64()
        .ok_or_else(|| anyhow::anyhow!("failed to convert decimal aggregate to u64: {}", value))
}
