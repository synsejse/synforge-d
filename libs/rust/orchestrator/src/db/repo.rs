use super::*;
use std::collections::{HashMap, HashSet};

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
            let rows = query
                .order((
                    build_artifacts::package_name.asc(),
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .limit(limit as i64)
                .offset(offset as i64)
                .select((
                    published_repo_files::artifact_id,
                    build_artifacts::job_id,
                    build_artifacts::package_name,
                    build_artifacts::mock_chroot,
                    published_repo_files::repo_path,
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
                    build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .filter(build_artifacts::package_name.eq(package_name.as_str()))
                .order((
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .select((
                    published_repo_files::artifact_id,
                    build_artifacts::job_id,
                    build_artifacts::package_name,
                    build_artifacts::mock_chroot,
                    published_repo_files::repo_path,
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
                    build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .order((
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .limit(limit as i64)
                .select((
                    published_repo_files::artifact_id,
                    build_artifacts::job_id,
                    build_artifacts::package_name,
                    build_artifacts::mock_chroot,
                    published_repo_files::repo_path,
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
    store
        .with_connection(move |conn| {
            let rows = published_repo_files::table
                .inner_join(
                    build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select((
                    build_artifacts::package_name,
                    build_artifacts::mock_chroot,
                    build_artifacts::job_id,
                    build_artifacts::size_bytes,
                ))
                .load::<(String, String, String, i64)>(conn)?;
            let mut by_target = HashMap::<String, (HashSet<String>, HashSet<String>, u64)>::new();
            for row in rows {
                let entry = by_target
                    .entry(row.1.clone())
                    .or_insert_with(|| (HashSet::new(), HashSet::new(), 0));
                entry.0.insert(row.0);
                entry.1.insert(row.2);
                entry.2 += row.3 as u64;
            }

            let mut targets = by_target
                .into_iter()
                .map(
                    |(mock_chroot, (packages, jobs, size_bytes))| RepoTargetSummary {
                        mock_chroot,
                        package_count: packages.len() as u64,
                        build_count: jobs.len() as u64,
                        size_bytes,
                    },
                )
                .collect::<Vec<_>>();
            targets.sort_by(|left, right| right.mock_chroot.cmp(&left.mock_chroot));
            Ok(targets)
        })
        .await
}

pub(super) async fn get_repo_distinct_counts(
    store: &DieselStore,
) -> anyhow::Result<(u64, u64, u64)> {
    store
        .with_connection(|conn| {
            let rows = published_repo_files::table
                .inner_join(
                    build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select((
                    build_artifacts::package_name,
                    build_artifacts::mock_chroot,
                    build_artifacts::job_id,
                ))
                .load::<(String, String, String)>(conn)?;

            let mut packages = HashSet::new();
            let mut targets = HashSet::new();
            let mut builds = HashSet::new();

            for (package_name, mock_chroot, job_id) in rows {
                packages.insert(package_name);
                targets.insert(mock_chroot);
                builds.insert(job_id);
            }

            Ok((
                packages.len() as u64,
                targets.len() as u64,
                builds.len() as u64,
            ))
        })
        .await
}

pub(super) async fn sum_published_repo_file_bytes(store: &DieselStore) -> anyhow::Result<u64> {
    store
        .with_connection(|conn| {
            let sizes = published_repo_files::table
                .inner_join(
                    build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
                )
                .select(build_artifacts::size_bytes)
                .load::<i64>(conn)?;
            Ok(sizes.into_iter().map(|value| value as u64).sum())
        })
        .await
}
