use super::*;

pub(super) async fn list_published_repo_files(
    store: &DieselStore,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    store
        .with_connection(move |conn| {
            let rows = published_repo_files::table
                .order((
                    published_repo_files::package_name.asc(),
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .limit(limit as i64)
                .offset(offset as i64)
                .select(PublishedRepoFileRecord::as_select())
                .load(conn)?;
            rows.into_iter().map(published_repo_file_from_record).collect()
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
                .filter(published_repo_files::package_name.eq(package_name.as_str()))
                .order((
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .select(PublishedRepoFileRecord::as_select())
                .load(conn)?;
            rows.into_iter().map(published_repo_file_from_record).collect()
        })
        .await
}
