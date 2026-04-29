use super::super::*;
use synforge_core::package::RepoTarget;

#[derive(Insertable)]
#[diesel(table_name = published_repo_files)]
pub(crate) struct NewPublishedRepoFileRecord {
    pub(crate) artifact_id: Uuid,
    pub(crate) published_at: OffsetDateTime,
}

type PublishedRepoFileRow = (
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

pub(crate) async fn load_published_repo_files_for_job(
    conn: &mut AsyncPgConnection,
    job_id: Uuid,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let rows = published_repo_files::table
        .inner_join(
            build_artifacts::table.on(published_repo_files::artifact_id.eq(build_artifacts::id)),
        )
        .left_join(
            artifact_signatures::table.on(build_artifacts::id.eq(artifact_signatures::artifact_id)),
        )
        .filter(build_artifacts::job_id.eq(job_id))
        .order(build_artifacts::file.asc())
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
        .load::<PublishedRepoFileRow>(conn)
        .await?;
    rows.into_iter()
        .map(published_repo_file_from_record)
        .collect()
}

pub(crate) fn published_repo_file_from_record(
    row: PublishedRepoFileRow,
) -> anyhow::Result<PublishedRepoFile> {
    let (
        artifact_id,
        job_id,
        package_name,
        mock_chroot,
        artifact_path,
        sha256,
        size_bytes,
        kind,
        published_at,
        signing_status,
        signing_error_message,
    ) = row;
    let path = build_published_repo_path(
        &package_name,
        &mock_chroot,
        job_id,
        Path::new(&artifact_path),
    )?;
    Ok(PublishedRepoFile {
        artifact_id,
        job_id,
        package_name,
        mock_chroot,
        path,
        sha256,
        size_bytes: size_bytes.max(0) as u64,
        kind,
        published_at,
        signing_status,
        signing_error_message,
    })
}

pub fn build_published_repo_path(
    package_name: &str,
    mock_chroot: &str,
    job_id: Uuid,
    artifact_path: &Path,
) -> anyhow::Result<PathBuf> {
    let target = RepoTarget::from_mock_chroot(mock_chroot)
        .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", mock_chroot))?;
    let file_name = artifact_path.file_name().ok_or_else(|| {
        anyhow::anyhow!(
            "artifact path {} has no filename for published repo path",
            artifact_path.display()
        )
    })?;
    Ok(target
        .repo_subdir()
        .join("packages")
        .join(package_name)
        .join("builds")
        .join(job_id.to_string())
        .join(file_name))
}
