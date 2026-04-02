use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel::upsert::excluded;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use synforge_core::{
    format_timestamp, now_utc, ArtifactKind, BuildArtifact, BuildEnvVar, BuildJob, BuildJobResponse,
    BuildStatus, BuildTrigger, PackageDefinition, PackageResponse, PackageRuntimeState,
    PublishedRepoFile, SpecSource,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::schema::{
    build_artifacts, build_jobs, build_logs, daemon_runtime_settings, packages, published_repo_files,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
const ENSURE_INDEXES_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_build_jobs_package_created_at
    ON build_jobs (package_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_jobs_package_status_created_at
    ON build_jobs (package_name, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_jobs_package_chroot_status_finished_at
    ON build_jobs (package_name, mock_chroot, status, finished_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_artifacts_job_id
    ON build_artifacts (job_id);
CREATE INDEX IF NOT EXISTS idx_published_repo_files_job_id
    ON published_repo_files (job_id);
CREATE INDEX IF NOT EXISTS idx_published_repo_files_package_published_at
    ON published_repo_files (package_name, published_at DESC);
"#;
const ENSURE_PACKAGE_HISTORY_COLUMN_SQL: &str = r#"
ALTER TABLE packages ADD COLUMN package_history_count BIGINT NOT NULL DEFAULT 3;
"#;

#[async_trait]
pub trait JobStore: Send + Sync {
    async fn list_packages(&self) -> anyhow::Result<Vec<PackageResponse>>;
    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>>;
    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()>;
    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()>;
    async fn get_last_successful_revision(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<Option<String>>;
    async fn has_active_job(&self, package_name: &str) -> anyhow::Result<bool>;
    async fn insert_job(&self, job: &BuildJob) -> anyhow::Result<()>;
    async fn set_job_running(&self, job_id: Uuid, worker_container_id: Option<&str>) -> anyhow::Result<()>;
    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
        logs_path: Option<&Path>,
    ) -> anyhow::Result<()>;
    async fn list_jobs(&self) -> anyhow::Result<Vec<BuildJobResponse>>;
    async fn list_jobs_for_package(&self, package_name: &str) -> anyhow::Result<Vec<BuildJobResponse>>;
    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn get_job_log_path(&self, job_id: Uuid) -> anyhow::Result<Option<PathBuf>>;
    async fn list_published_repo_files(&self) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn list_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn list_published_repo_files_for_package(&self, package_name: &str) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()>;
    async fn get_public_base_url_override(&self) -> anyhow::Result<Option<String>>;
    async fn set_public_base_url_override(&self, value: &str) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct DieselStore {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DieselStore {
    pub async fn new(database_path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let database_url = database_path.to_string_lossy().to_string();
        let pool = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_url);
            let pool = Pool::builder().max_size(5).build(manager)?;
            let mut conn = pool.get()?;
            conn.batch_execute("PRAGMA journal_mode = WAL;")?;
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!("failed to run diesel migrations: {}", error))?;
            let _ = conn.batch_execute(ENSURE_PACKAGE_HISTORY_COLUMN_SQL);
            conn.batch_execute(ENSURE_INDEXES_SQL)?;
            Ok(pool)
        })
        .await
        .context("diesel store initialization task failed")??;
        Ok(Self { pool })
    }

    async fn with_connection<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut SqliteConnection) -> anyhow::Result<T> + Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            conn.batch_execute("PRAGMA journal_mode = WAL;")?;
            f(&mut conn)
        })
        .await
        .context("diesel database task failed")?
    }
}

#[async_trait]
impl JobStore for DieselStore {
    async fn list_packages(&self) -> anyhow::Result<Vec<PackageResponse>> {
        self.with_connection(|conn| {
            let rows = packages::table
                .order(packages::name.asc())
                .select(PackageRecord::as_select())
                .load(conn)?;
            rows.into_iter()
                .map(|record| package_response_from_record(conn, record))
                .collect()
        })
        .await
    }

    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>> {
        let package_name = package_name.to_string();
        self.with_connection(move |conn| {
            let row = packages::table
                .find(package_name)
                .select(PackageRecord::as_select())
                .first(conn)
                .optional()?;
            row.map(|record| package_response_from_record(conn, record)).transpose()
        })
        .await
    }

    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()> {
        let package = package.clone();
        self.with_connection(move |conn| {
            let spec_path = package.spec_path.to_string_lossy().to_string();
            let build_env_json = serde_json::to_string(&package.build_env)?;
            let mock_chroots_json = serde_json::to_string(&package.mock_chroots)?;
            let new_row = NewPackageRecord {
                name: package.name.as_str(),
                description: package.description.as_str(),
                enabled: package.enabled,
                repo_subdir: package.repo_subdir.as_str(),
                publish_srpm: package.publish_srpm,
                mock_chroots_json: mock_chroots_json.as_str(),
                source_repo_url: package.source.repo_url.as_str(),
                source_spec_path: package.source.spec_path.as_str(),
                source_poll: package.source.poll,
                poll_interval_seconds: package.poll_interval_seconds as i64,
                build_timeout_seconds: package.build_timeout_seconds as i64,
                package_history_count: package.package_history_count as i64,
                build_env_json: build_env_json.as_str(),
                spec_path: spec_path.as_str(),
                version: package.version.as_str(),
                release: package.release.as_str(),
            };
            diesel::insert_into(packages::table)
                .values(&new_row)
                .on_conflict(packages::name)
                .do_update()
                .set((
                    packages::description.eq(excluded(packages::description)),
                    packages::enabled.eq(excluded(packages::enabled)),
                    packages::repo_subdir.eq(excluded(packages::repo_subdir)),
                    packages::publish_srpm.eq(excluded(packages::publish_srpm)),
                    packages::mock_chroots_json.eq(excluded(packages::mock_chroots_json)),
                    packages::source_repo_url.eq(excluded(packages::source_repo_url)),
                    packages::source_spec_path.eq(excluded(packages::source_spec_path)),
                    packages::source_poll.eq(excluded(packages::source_poll)),
                    packages::poll_interval_seconds.eq(excluded(packages::poll_interval_seconds)),
                    packages::build_timeout_seconds.eq(excluded(packages::build_timeout_seconds)),
                    packages::package_history_count.eq(excluded(packages::package_history_count)),
                    packages::build_env_json.eq(excluded(packages::build_env_json)),
                    packages::spec_path.eq(excluded(packages::spec_path)),
                    packages::version.eq(excluded(packages::version)),
                    packages::release.eq(excluded(packages::release)),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()> {
        let package_name = package_name.to_string();
        self.with_connection(move |conn| {
            diesel::delete(packages::table.find(package_name)).execute(conn)?;
            Ok(())
        })
        .await
    }

    async fn get_last_successful_revision(&self, package_name: &str, mock_chroot: &str) -> anyhow::Result<Option<String>> {
        let package_name = package_name.to_string();
        let mock_chroot = mock_chroot.to_string();
        self.with_connection(move |conn| {
            Ok(build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(build_jobs::mock_chroot.eq(mock_chroot.as_str()))
                .filter(build_jobs::status.eq(build_status_value(BuildStatus::Succeeded)))
                .order(build_jobs::finished_at.desc())
                .select(build_jobs::revision)
                .first::<String>(conn)
                .optional()?
            )
        })
        .await
    }

    async fn insert_job(&self, job: &BuildJob) -> anyhow::Result<()> {
        let job = job.clone();
        self.with_connection(move |conn| {
            let id = job.id.to_string();
            let spec_path = job.spec_path.to_string_lossy().to_string();
            let new_job = NewJobRecord {
                id: id.as_str(),
                package_name: job.package_name.as_str(),
                mock_chroot: job.mock_chroot.as_str(),
                revision: job.revision.as_str(),
                trigger: build_trigger_value(job.trigger),
                status: build_status_value(job.status),
                spec_path: spec_path.as_str(),
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

    async fn has_active_job(&self, package_name: &str) -> anyhow::Result<bool> {
        let package_name = package_name.to_string();
        self.with_connection(move |conn| {
            let active_job = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .filter(
                    build_jobs::status
                        .eq(build_status_value(BuildStatus::Pending))
                        .or(build_jobs::status.eq(build_status_value(BuildStatus::Running))),
                )
                .select(build_jobs::id)
                .first::<String>(conn)
                .optional()?;
            Ok(active_job.is_some())
        })
        .await
    }

    async fn set_job_running(&self, job_id: Uuid, worker_container_id: Option<&str>) -> anyhow::Result<()> {
        let job_id = job_id.to_string();
        let now = format_timestamp(now_utc());
        let worker_container_id = worker_container_id.map(ToOwned::to_owned);
        self.with_connection(move |conn| {
            diesel::update(build_jobs::table.find(job_id.as_str()))
                .set((
                    build_jobs::status.eq(build_status_value(BuildStatus::Running)),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::worker_container_id.eq(worker_container_id.as_deref()),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
        logs_path: Option<&Path>,
    ) -> anyhow::Result<()> {
        let job_id = job_id.to_string();
        let status_value = build_status_value(status).to_string();
        let error_message = error_message.map(ToOwned::to_owned);
        let logs_path = logs_path.map(|path| path.to_path_buf());
        let artifacts = artifacts.to_vec();
        let published_files = published_files.to_vec();
        self.with_connection(move |conn| {
            conn.transaction::<(), diesel::result::Error, _>(|conn| {
                let now = format_timestamp(now_utc());
                let job_row = build_jobs::table
                    .find(job_id.as_str())
                    .select(JobRecord::as_select())
                    .first(conn)?;

                diesel::update(build_jobs::table.find(job_id.as_str()))
                    .set((
                        build_jobs::status.eq(status_value.as_str()),
                        build_jobs::updated_at.eq(now.as_str()),
                        build_jobs::finished_at.eq(Some(now.as_str())),
                        build_jobs::error_message.eq(error_message.as_deref()),
                    ))
                    .execute(conn)?;

                diesel::delete(build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())))
                    .execute(conn)?;

                if !artifacts.is_empty() {
                    let rows = artifacts
                        .iter()
                        .map(|artifact| NewArtifactRecord {
                            job_id: job_id.clone(),
                            package_name: job_row.package_name.clone(),
                            mock_chroot: job_row.mock_chroot.clone(),
                            arch: artifact.arch.clone(),
                            path: artifact.path.to_string_lossy().to_string(),
                            relative_repo_path: artifact.relative_repo_path.to_string_lossy().to_string(),
                            sha256: artifact.sha256.clone(),
                            size_bytes: artifact.size_bytes as i64,
                            kind: artifact_kind_value(artifact.kind).to_string(),
                        })
                        .collect::<Vec<_>>();
                    diesel::insert_into(build_artifacts::table)
                        .values(&rows)
                        .execute(conn)?;
                }

                if let Some(logs_path) = &logs_path {
                    let log_path = logs_path.to_string_lossy().to_string();
                    let log_row = NewBuildLogRecord {
                        job_id: job_id.as_str(),
                        log_path: log_path.as_str(),
                        updated_at: now.as_str(),
                    };
                    diesel::insert_into(build_logs::table)
                        .values(&log_row)
                        .on_conflict(build_logs::job_id)
                        .do_update()
                        .set((
                            build_logs::log_path.eq(log_path.as_str()),
                            build_logs::updated_at.eq(now.as_str()),
                        ))
                        .execute(conn)?;
                }

                diesel::delete(
                    published_repo_files::table
                        .filter(published_repo_files::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;

                if !published_files.is_empty() {
                    let rows = published_files
                        .iter()
                        .map(|file| NewPublishedRepoFileRecord {
                            job_id: job_id.clone(),
                            package_name: file.package_name.clone(),
                            mock_chroot: file.mock_chroot.clone(),
                            arch: file.arch.clone(),
                            repo_path: file.repo_path.to_string_lossy().to_string(),
                            sha256: file.sha256.clone(),
                            size_bytes: file.size_bytes as i64,
                            kind: artifact_kind_value(file.kind).to_string(),
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

    async fn list_jobs(&self) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.with_connection(|conn| {
            let rows = build_jobs::table
                .order(build_jobs::created_at.desc())
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
    }

    async fn list_jobs_for_package(&self, package_name: &str) -> anyhow::Result<Vec<BuildJobResponse>> {
        let package_name = package_name.to_string();
        self.with_connection(move |conn| {
            let rows = build_jobs::table
                .filter(build_jobs::package_name.eq(package_name.as_str()))
                .order(build_jobs::created_at.desc())
                .select(JobRecord::as_select())
                .load(conn)?;
            load_job_responses(conn, rows)
        })
        .await
    }

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        let job_id = job_id.to_string();
        self.with_connection(move |conn| {
            let row = build_jobs::table
                .find(job_id.as_str())
                .select(JobRecord::as_select())
                .first(conn)
                .optional()?;
            let artifacts = load_artifacts_map(conn)?;
            row.map(|row| build_job_response_from_row(row, &artifacts))
            .transpose()
        })
        .await
    }

    async fn get_job_log_path(&self, job_id: Uuid) -> anyhow::Result<Option<PathBuf>> {
        let job_id = job_id.to_string();
        self.with_connection(move |conn| {
            Ok(build_logs::table
                .find(job_id.as_str())
                .select(build_logs::log_path)
                .first::<String>(conn)
                .optional()?
                .map(PathBuf::from))
        })
        .await
    }

    async fn list_published_repo_files(&self) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.with_connection(move |conn| {
            let rows = published_repo_files::table
                .order((
                    published_repo_files::package_name.asc(),
                    published_repo_files::published_at.desc(),
                    published_repo_files::repo_path.asc(),
                ))
                .select(PublishedRepoFileRecord::as_select())
                .load(conn)?;
            rows.into_iter().map(published_repo_file_from_record).collect()
        })
        .await
    }

    async fn list_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<PublishedRepoFile>> {
        let job_id = job_id.to_string();
        self.with_connection(move |conn| load_published_repo_files_for_job(conn, job_id.as_str()))
            .await
    }

    async fn list_published_repo_files_for_package(&self, package_name: &str) -> anyhow::Result<Vec<PublishedRepoFile>> {
        let package_name = package_name.to_string();
        self.with_connection(move |conn| {
            let rows = published_repo_files::table
                .filter(published_repo_files::package_name.eq(package_name.as_str()))
                .order((published_repo_files::published_at.desc(), published_repo_files::repo_path.asc()))
                .select(PublishedRepoFileRecord::as_select())
                .load(conn)?;
            rows.into_iter().map(published_repo_file_from_record).collect()
        })
        .await
    }

    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        let job_id = job_id.to_string();
        self.with_connection(move |conn| {
            let artifacts = load_artifacts_map(conn)?;
            let row = build_jobs::table
                .find(job_id.as_str())
                .select(JobRecord::as_select())
                .first(conn)
                .optional()?;
            let Some(row) = row else {
                return Ok(None);
            };
            let response = build_job_response_from_row(row, &artifacts)?;
            if matches!(response.job.status, BuildStatus::Pending | BuildStatus::Running) {
                return Err(anyhow::anyhow!(
                    "cannot delete a pending or running job"
                ));
            }

            conn.transaction::<(), anyhow::Error, _>(|conn| {
                diesel::delete(build_artifacts::table.filter(build_artifacts::job_id.eq(job_id.as_str())))
                    .execute(conn)?;
                diesel::delete(build_logs::table.filter(build_logs::job_id.eq(job_id.as_str())))
                    .execute(conn)?;
                diesel::delete(
                    published_repo_files::table.filter(published_repo_files::job_id.eq(job_id.as_str())),
                )
                .execute(conn)?;
                diesel::delete(build_jobs::table.find(job_id.as_str())).execute(conn)?;

                Ok(())
            })?;

            Ok(Some(response))
        })
        .await
    }

    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()> {
        let message = message.to_string();
        self.with_connection(move |conn| {
            let now = format_timestamp(now_utc());
            diesel::update(build_jobs::table.filter(build_jobs::finished_at.is_null()))
                .set((
                    build_jobs::status.eq(build_status_value(BuildStatus::Failed)),
                    build_jobs::updated_at.eq(now.as_str()),
                    build_jobs::finished_at.eq(Some(now.as_str())),
                    build_jobs::error_message.eq(Some(message.as_str())),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    async fn get_public_base_url_override(&self) -> anyhow::Result<Option<String>> {
        self.with_connection(|conn| {
            let value = daemon_runtime_settings::table
                .find(1)
                .select(daemon_runtime_settings::public_base_url)
                .first::<Option<String>>(conn)
                .optional()?
                .flatten()
                .filter(|value| !value.trim().is_empty());
            Ok(value)
        })
        .await
    }

    async fn set_public_base_url_override(&self, value: &str) -> anyhow::Result<()> {
        let value = value.trim().to_string();
        self.with_connection(move |conn| {
            let row = RuntimeSettingsRecord {
                id: 1,
                public_base_url: Some(value.as_str()),
            };
            diesel::insert_into(daemon_runtime_settings::table)
                .values(&row)
                .on_conflict(daemon_runtime_settings::id)
                .do_update()
                .set(daemon_runtime_settings::public_base_url.eq(row.public_base_url))
                .execute(conn)?;
            Ok(())
        })
        .await
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = packages)]
struct PackageRecord {
    name: String,
    description: String,
    enabled: bool,
    repo_subdir: String,
    publish_srpm: bool,
    mock_chroots_json: String,
    source_repo_url: String,
    source_spec_path: String,
    source_poll: bool,
    poll_interval_seconds: i64,
    build_timeout_seconds: i64,
    package_history_count: i64,
    build_env_json: String,
    spec_path: String,
    version: String,
    release: String,
}

#[derive(Insertable)]
#[diesel(table_name = packages)]
struct NewPackageRecord<'a> {
    name: &'a str,
    description: &'a str,
    enabled: bool,
    repo_subdir: &'a str,
    publish_srpm: bool,
    mock_chroots_json: &'a str,
    source_repo_url: &'a str,
    source_spec_path: &'a str,
    source_poll: bool,
    poll_interval_seconds: i64,
    build_timeout_seconds: i64,
    package_history_count: i64,
    build_env_json: &'a str,
    spec_path: &'a str,
    version: &'a str,
    release: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_jobs)]
struct JobRecord {
    id: String,
    package_name: String,
    mock_chroot: String,
    revision: String,
    trigger: String,
    status: String,
    spec_path: String,
    worker_container_id: Option<String>,
    created_at: String,
    updated_at: String,
    finished_at: Option<String>,
    error_message: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = build_jobs)]
struct NewJobRecord<'a> {
    id: &'a str,
    package_name: &'a str,
    mock_chroot: &'a str,
    revision: &'a str,
    trigger: &'a str,
    status: &'a str,
    spec_path: &'a str,
    worker_container_id: Option<&'a str>,
    created_at: String,
    updated_at: String,
    finished_at: Option<String>,
    error_message: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_artifacts)]
struct ArtifactRecord {
    job_id: String,
    package_name: String,
    mock_chroot: String,
    arch: String,
    path: String,
    relative_repo_path: String,
    sha256: String,
    size_bytes: i64,
    kind: String,
}

#[derive(Insertable)]
#[diesel(table_name = build_artifacts)]
struct NewArtifactRecord {
    job_id: String,
    package_name: String,
    mock_chroot: String,
    arch: String,
    path: String,
    relative_repo_path: String,
    sha256: String,
    size_bytes: i64,
    kind: String,
}

#[derive(Insertable)]
#[diesel(table_name = build_logs)]
struct NewBuildLogRecord<'a> {
    job_id: &'a str,
    log_path: &'a str,
    updated_at: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = published_repo_files)]
struct PublishedRepoFileRecord {
    job_id: String,
    package_name: String,
    mock_chroot: String,
    arch: String,
    repo_path: String,
    sha256: String,
    size_bytes: i64,
    kind: String,
    published_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = published_repo_files)]
struct NewPublishedRepoFileRecord {
    job_id: String,
    package_name: String,
    mock_chroot: String,
    arch: String,
    repo_path: String,
    sha256: String,
    size_bytes: i64,
    kind: String,
    published_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = daemon_runtime_settings)]
struct RuntimeSettingsRecord<'a> {
    id: i32,
    public_base_url: Option<&'a str>,
}

fn package_response_from_record(
    conn: &mut SqliteConnection,
    record: PackageRecord,
) -> anyhow::Result<PackageResponse> {
    let package = PackageDefinition {
        name: record.name.clone(),
        description: record.description,
        enabled: record.enabled,
        repo_subdir: record.repo_subdir,
        publish_srpm: record.publish_srpm,
        mock_chroots: serde_json::from_str::<Vec<String>>(&record.mock_chroots_json)
            .unwrap_or_default(),
        source: SpecSource {
            repo_url: record.source_repo_url,
            spec_path: record.source_spec_path,
            poll: record.source_poll,
        },
        poll_interval_seconds: record.poll_interval_seconds as u64,
        build_timeout_seconds: record.build_timeout_seconds as u64,
        package_history_count: record.package_history_count as u64,
        build_env: serde_json::from_str::<Vec<BuildEnvVar>>(&record.build_env_json)
            .unwrap_or_default(),
        spec_path: PathBuf::from(record.spec_path),
        version: record.version,
        release: record.release,
    };
    let state = derive_package_state(conn, &record.name)?;
    Ok(PackageResponse {
        package,
        state,
    })
}

fn derive_package_state(
    conn: &mut SqliteConnection,
    package_name: &str,
) -> anyhow::Result<PackageRuntimeState> {
    let last_success = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(build_jobs::status.eq(build_status_value(BuildStatus::Succeeded)))
        .order(build_jobs::finished_at.desc())
        .select((build_jobs::id, build_jobs::revision))
        .first::<(String, String)>(conn)
        .optional()?;

    let active_job = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(
            build_jobs::status
                .eq(build_status_value(BuildStatus::Pending))
                .or(build_jobs::status.eq(build_status_value(BuildStatus::Running))),
        )
        .order(build_jobs::created_at.desc())
        .select(build_jobs::id)
        .first::<String>(conn)
        .optional()?;

    Ok(PackageRuntimeState {
        last_revision: last_success.as_ref().map(|(_, revision)| revision.clone()),
        last_successful_build_id: last_success
            .as_ref()
            .map(|(id, _)| Uuid::parse_str(id))
            .transpose()?,
        active_job_id: active_job
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()?,
    })
}

fn load_artifacts_map(conn: &mut SqliteConnection) -> anyhow::Result<HashMap<Uuid, Vec<BuildArtifact>>> {
    let rows = build_artifacts::table
        .select(ArtifactRecord::as_select())
        .load(conn)?;
    let mut map: HashMap<Uuid, Vec<BuildArtifact>> = HashMap::new();
    for row in rows {
        let job_id = Uuid::parse_str(&row.job_id)?;
        map.entry(job_id).or_default().push(BuildArtifact {
            package_name: row.package_name,
            mock_chroot: row.mock_chroot,
            arch: row.arch,
            path: PathBuf::from(row.path),
            relative_repo_path: PathBuf::from(row.relative_repo_path),
            sha256: row.sha256,
            size_bytes: row.size_bytes as u64,
            kind: parse_artifact_kind(&row.kind),
        });
    }
    Ok(map)
}

fn load_job_responses(
    conn: &mut SqliteConnection,
    rows: Vec<JobRecord>,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let artifacts = load_artifacts_map(conn)?;
    rows.into_iter()
        .map(|row| build_job_response_from_row(row, &artifacts))
        .collect()
}

fn build_job_response_from_row(
    row: JobRecord,
    artifacts: &HashMap<Uuid, Vec<BuildArtifact>>,
) -> anyhow::Result<BuildJobResponse> {
    let job = row_to_build_job(row)?;
    Ok(BuildJobResponse {
        artifacts: artifacts.get(&job.id).cloned().unwrap_or_default(),
        job,
    })
}

fn row_to_build_job(row: JobRecord) -> anyhow::Result<BuildJob> {
    Ok(BuildJob {
        id: Uuid::parse_str(&row.id)?,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        revision: row.revision,
        trigger: parse_build_trigger(&row.trigger),
        status: parse_build_status(&row.status),
        spec_path: PathBuf::from(row.spec_path),
        worker_container_id: row.worker_container_id,
        created_at: parse_timestamp(&row.created_at)?,
        updated_at: parse_timestamp(&row.updated_at)?,
        finished_at: row.finished_at.as_deref().map(parse_timestamp).transpose()?,
        error_message: row.error_message,
    })
}

fn artifact_kind_value(value: ArtifactKind) -> &'static str {
    match value {
        ArtifactKind::Rpm => "rpm",
        ArtifactKind::Srpm => "srpm",
        ArtifactKind::Log => "log",
        ArtifactKind::Other => "other",
    }
}

fn build_trigger_value(value: BuildTrigger) -> &'static str {
    match value {
        BuildTrigger::Poll => "poll",
        BuildTrigger::ManualRefresh => "manual_refresh",
        BuildTrigger::ManualRebuild => "manual_rebuild",
        BuildTrigger::Api => "api",
    }
}

fn build_status_value(value: BuildStatus) -> &'static str {
    match value {
        BuildStatus::Pending => "pending",
        BuildStatus::Running => "running",
        BuildStatus::Succeeded => "succeeded",
        BuildStatus::Failed => "failed",
        BuildStatus::TimedOut => "timed_out",
    }
}

fn parse_artifact_kind(value: &str) -> ArtifactKind {
    match value {
        "srpm" => ArtifactKind::Srpm,
        "log" => ArtifactKind::Log,
        "other" => ArtifactKind::Other,
        _ => ArtifactKind::Rpm,
    }
}

fn parse_build_trigger(value: &str) -> BuildTrigger {
    match value {
        "manualrefresh" | "manual_refresh" => BuildTrigger::ManualRefresh,
        "manualrebuild" | "manual_rebuild" => BuildTrigger::ManualRebuild,
        "api" => BuildTrigger::Api,
        _ => BuildTrigger::Poll,
    }
}

fn parse_build_status(value: &str) -> BuildStatus {
    match value {
        "running" => BuildStatus::Running,
        "succeeded" => BuildStatus::Succeeded,
        "failed" => BuildStatus::Failed,
        "timedout" | "timed_out" => BuildStatus::TimedOut,
        _ => BuildStatus::Pending,
    }
}

fn parse_timestamp(value: &str) -> anyhow::Result<OffsetDateTime> {
    Ok(OffsetDateTime::parse(value, &Rfc3339)?)
}

fn load_published_repo_files_for_job(
    conn: &mut SqliteConnection,
    job_id: &str,
) -> anyhow::Result<Vec<PublishedRepoFile>> {
    let rows = published_repo_files::table
        .filter(published_repo_files::job_id.eq(job_id))
        .order(published_repo_files::repo_path.asc())
        .select(PublishedRepoFileRecord::as_select())
        .load(conn)?;
    rows.into_iter().map(published_repo_file_from_record).collect()
}

fn published_repo_file_from_record(row: PublishedRepoFileRecord) -> anyhow::Result<PublishedRepoFile> {
    Ok(PublishedRepoFile {
        job_id: Uuid::parse_str(&row.job_id)?,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        arch: row.arch,
        repo_path: PathBuf::from(row.repo_path),
        sha256: row.sha256,
        size_bytes: row.size_bytes.max(0) as u64,
        kind: parse_artifact_kind(&row.kind),
        published_at: parse_timestamp(&row.published_at)?,
    })
}
