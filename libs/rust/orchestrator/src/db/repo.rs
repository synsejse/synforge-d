use super::*;
use bigdecimal::{BigDecimal, ToPrimitive};
#[allow(deprecated)]
use diesel::dsl::{count_distinct, sum};

type PublishedRepoRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    ArtifactKind,
    String,
);

pub(super) async fn list_published_repo_files(
    store: &DieselStore,
    limit: usize,
    offset: usize,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    kind: Option<ArtifactKind>,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    store
        .with_connection(move |conn| {
            let mut query = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
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
                ))
                .load::<PublishedRepoRow>(conn)?;
            rows.into_iter()
                .map(published_repo_file_from_record)
                .collect()
        })
        .await
}

pub(super) async fn list_published_repo_files_for_job(
    store: &DieselStore,
    job_id: Uuid,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let job_id = job_id.to_string();
    store
        .with_connection(move |conn| load_published_repo_files_for_job(conn, job_id.as_str()))
        .await
}

pub(super) async fn list_published_repo_files_for_package(
    store: &DieselStore,
    package_name: &str,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let package_name = package_name.to_string();
    store
        .with_connection(move |conn| {
            let rows = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
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
                ))
                .load::<PublishedRepoRow>(conn)?;
            rows.into_iter()
                .map(published_repo_file_from_record)
                .collect()
        })
        .await
}

pub(super) async fn count_published_repo_files(
    store: &DieselStore,
    package_name: Option<String>,
    mock_chroot: Option<String>,
    kind: Option<ArtifactKind>,
) -> anyhow::Result<u64> {
    store
        .with_connection(move |conn| {
            let mut query = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
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
            let count = query.count().get_result::<i64>(conn)?;
            Ok(count as u64)
        })
        .await
}

pub(super) async fn list_recent_published_repo_files(
    store: &DieselStore,
    limit: usize,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    store
        .with_connection(move |conn| {
            let rows = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
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
                ))
                .load::<PublishedRepoRow>(conn)?;
            rows.into_iter()
                .map(published_repo_file_from_record)
                .collect()
        })
        .await
}

pub(super) async fn list_repo_target_summaries(
    store: &DieselStore,
) -> anyhow::Result<Vec<RepoTargetSummary>> {
    #[allow(deprecated)]
    store
        .with_connection(move |conn| {
            let rows = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .group_by(build_artifacts::mock_chroot)
                .order(build_artifacts::mock_chroot.desc())
                .select((
                    build_artifacts::mock_chroot,
                    count_distinct(build_artifacts::package_name),
                    count_distinct(build_artifacts::job_id),
                    sum(build_artifacts::size_bytes),
                ))
                .load::<(String, i64, i64, Option<BigDecimal>)>(conn)?;

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
        })
        .await
}

pub(super) async fn get_repo_distinct_counts(
    store: &DieselStore,
) -> anyhow::Result<(u64, u64, u64)> {
    #[allow(deprecated)]
    store
        .with_connection(|conn| {
            let package_count = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select(count_distinct(build_artifacts::package_name))
                .first::<i64>(conn)? as u64;
            let target_count = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select(count_distinct(build_artifacts::mock_chroot))
                .first::<i64>(conn)? as u64;
            let build_count = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select(count_distinct(build_artifacts::job_id))
                .first::<i64>(conn)? as u64;

            Ok((package_count, target_count, build_count))
        })
        .await
}

pub(super) async fn sum_published_repo_file_bytes(store: &DieselStore) -> anyhow::Result<u64> {
    store
        .with_connection(|conn| {
            let total_size = published_repo_files::table
                .inner_join(
                    build_artifacts::table
                        .on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select(sum(build_artifacts::size_bytes))
                .first::<Option<BigDecimal>>(conn)?;
            Ok(total_size
                .map(decimal_to_u64)
                .transpose()?
                .unwrap_or_default())
        })
        .await
}

fn decimal_to_u64(value: BigDecimal) -> anyhow::Result<u64> {
    value
        .to_u64()
        .ok_or_else(|| anyhow::anyhow!("failed to convert decimal aggregate to u64: {}", value))
}
