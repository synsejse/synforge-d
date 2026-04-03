mod job;
mod package;
mod repo;
mod user;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel::upsert::excluded;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use synforge_core::{
    api::{BuildJobResponse, PackageResponse, RepoTargetSummary},
    model::{
        format_timestamp, now_utc, ArtifactKind, BuildArtifact, BuildJob, BuildStatus, BuildTrigger,
        PackageRuntimeState, PublishedRepoFile, UserAccount, UserPermission, UserRepoMetrics,
        UserSummary,
    },
    package::{BuildEnvVar, PackageDefinition, SpecSource},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::schema::{
    build_artifacts, build_jobs, build_logs, packages, published_repo_files, user_permissions,
    user_repo_metrics, users,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[async_trait]
pub trait JobStore: Send + Sync {
    async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>>;
    async fn count_packages(&self, search: Option<String>, enabled: Option<bool>) -> anyhow::Result<u64>;
    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>>;
    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()>;
    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()>;
    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>>;
    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool>;
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
    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>>;
    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64>;
    async fn list_jobs_for_package(&self, package_name: &str) -> anyhow::Result<Vec<BuildJobResponse>>;
    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64>;
    async fn list_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn list_published_repo_files_for_package(&self, package_name: &str) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn list_recent_published_repo_files(&self, limit: usize) -> anyhow::Result<Vec<PublishedRepoFile>>;
    async fn list_repo_target_summaries(&self) -> anyhow::Result<Vec<RepoTargetSummary>>;
    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)>;
    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64>;
    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>>;
    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()>;
    async fn list_prunable_successful_job_ids(
        &self,
        package_name: &str,
        mock_chroot: &str,
        keep: usize,
    ) -> anyhow::Result<Vec<Uuid>>;
    async fn user_count(&self) -> anyhow::Result<u64>;
    async fn list_users(&self) -> anyhow::Result<Vec<UserSummary>>;
    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>>;
    async fn get_user_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserSummary>>;
    async fn get_user_auth_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserAuthRecord>>;
    async fn create_user(
        &self,
        handle: &str,
        display_name: &str,
        password_hash: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<UserSummary>;
    async fn update_user(
        &self,
        user_id: Uuid,
        handle: &str,
        display_name: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<Option<UserSummary>>;
    async fn update_user_password(&self, user_id: Uuid, password_hash: &str) -> anyhow::Result<bool>;
    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>>;
    async fn increment_user_download_bytes(&self, user_id: Uuid, bytes: u64) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct DieselStore {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DieselStore {
    pub async fn new(database_path: &Path, pool_size: u32) -> anyhow::Result<Self> {
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let database_url = database_path.to_string_lossy().to_string();
        let pool = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_url);
            let pool = Pool::builder().max_size(pool_size).build(manager)?;
            let mut conn = pool.get()?;
            diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!("failed to run diesel migrations: {}", error))?;
            Ok(pool)
        })
        .await
        .context("diesel store initialization task failed")??;
        Ok(Self { pool })
    }

    pub(crate) async fn with_connection<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut SqliteConnection) -> anyhow::Result<T> + Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            f(&mut conn)
        })
        .await
        .context("diesel database task failed")?
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        self.with_connection(|conn| {
            packages::table.count().get_result::<i64>(conn)?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl JobStore for DieselStore {
    async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>> {
        package::list_packages(self, limit, offset, search, enabled).await
    }

    async fn count_packages(&self, search: Option<String>, enabled: Option<bool>) -> anyhow::Result<u64> {
        package::count_packages(self, search, enabled).await
    }

    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>> {
        package::get_package(self, package_name).await
    }

    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()> {
        package::upsert_package(self, package).await
    }

    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()> {
        package::remove_package(self, package_name).await
    }

    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>> {
        job::get_last_successful_revision(self, package_name, mock_chroot).await
    }

    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        job::has_active_job_for_target(self, package_name, mock_chroot).await
    }

    async fn insert_job(&self, job: &BuildJob) -> anyhow::Result<()> {
        job::insert_job(self, job).await
    }

    async fn set_job_running(&self, job_id: Uuid, worker_container_id: Option<&str>) -> anyhow::Result<()> {
        job::set_job_running(self, job_id, worker_container_id).await
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
        job::finish_job(self, job_id, status, error_message, artifacts, published_files, logs_path).await
    }

    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs(self, limit, offset, status, package_name, mock_chroot).await
    }

    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64> {
        job::count_jobs(self, status, package_name, mock_chroot).await
    }

    async fn list_jobs_for_package(&self, package_name: &str) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs_for_package(self, package_name).await
    }

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        job::get_job(self, job_id).await
    }

    async fn list_published_repo_files(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files(self, limit, offset, package_name, mock_chroot, kind).await
    }

    async fn count_published_repo_files(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        kind: Option<ArtifactKind>,
    ) -> anyhow::Result<u64> {
        repo::count_published_repo_files(self, package_name, mock_chroot, kind).await
    }

    async fn list_published_repo_files_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_job(self, job_id).await
    }

    async fn list_published_repo_files_for_package(&self, package_name: &str) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_package(self, package_name).await
    }

    async fn list_recent_published_repo_files(&self, limit: usize) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_recent_published_repo_files(self, limit).await
    }

    async fn list_repo_target_summaries(&self) -> anyhow::Result<Vec<RepoTargetSummary>> {
        repo::list_repo_target_summaries(self).await
    }

    async fn get_repo_distinct_counts(&self) -> anyhow::Result<(u64, u64, u64)> {
        repo::get_repo_distinct_counts(self).await
    }

    async fn sum_published_repo_file_bytes(&self) -> anyhow::Result<u64> {
        repo::sum_published_repo_file_bytes(self).await
    }

    async fn delete_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        job::delete_job(self, job_id).await
    }

    async fn abort_unfinished_jobs(&self, message: &str) -> anyhow::Result<()> {
        job::abort_unfinished_jobs(self, message).await
    }

    async fn list_prunable_successful_job_ids(
        &self,
        package_name: &str,
        mock_chroot: &str,
        keep: usize,
    ) -> anyhow::Result<Vec<Uuid>> {
        job::list_prunable_successful_job_ids(self, package_name, mock_chroot, keep).await
    }

    async fn user_count(&self) -> anyhow::Result<u64> {
        user::user_count(self).await
    }

    async fn list_users(&self) -> anyhow::Result<Vec<UserSummary>> {
        user::list_users(self).await
    }

    async fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        user::get_user(self, user_id).await
    }

    async fn get_user_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserSummary>> {
        user::get_user_by_handle(self, handle).await
    }

    async fn get_user_auth_by_handle(&self, handle: &str) -> anyhow::Result<Option<UserAuthRecord>> {
        user::get_user_auth_by_handle(self, handle).await
    }

    async fn create_user(
        &self,
        handle: &str,
        display_name: &str,
        password_hash: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<UserSummary> {
        user::create_user(self, handle, display_name, password_hash, active, permissions).await
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        handle: &str,
        display_name: &str,
        active: bool,
        permissions: &[UserPermission],
    ) -> anyhow::Result<Option<UserSummary>> {
        user::update_user(self, user_id, handle, display_name, active, permissions).await
    }

    async fn update_user_password(&self, user_id: Uuid, password_hash: &str) -> anyhow::Result<bool> {
        user::update_user_password(self, user_id, password_hash).await
    }

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        user::delete_user(self, user_id).await
    }

    async fn increment_user_download_bytes(&self, user_id: Uuid, bytes: u64) -> anyhow::Result<()> {
        user::increment_user_download_bytes(self, user_id, bytes).await
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = packages)]
pub(crate) struct PackageRecord {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) enabled: bool,
    pub(crate) repo_subdir: String,
    pub(crate) publish_srpm: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: String,
    pub(crate) source_repo_url: String,
    pub(crate) source_spec_path: String,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) build_env_json: String,
    pub(crate) spec_path: String,
    pub(crate) version: String,
    pub(crate) release: String,
}

#[derive(Insertable)]
#[diesel(table_name = packages)]
pub(crate) struct NewPackageRecord<'a> {
    pub(crate) name: &'a str,
    pub(crate) description: &'a str,
    pub(crate) enabled: bool,
    pub(crate) repo_subdir: &'a str,
    pub(crate) publish_srpm: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: &'a str,
    pub(crate) source_repo_url: &'a str,
    pub(crate) source_spec_path: &'a str,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) build_env_json: &'a str,
    pub(crate) spec_path: &'a str,
    pub(crate) version: &'a str,
    pub(crate) release: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_jobs)]
pub(crate) struct JobRecord {
    pub(crate) id: String,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) revision: String,
    pub(crate) trigger: BuildTrigger,
    pub(crate) status: BuildStatus,
    pub(crate) spec_path: String,
    pub(crate) worker_container_id: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) finished_at: Option<String>,
    pub(crate) error_message: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = build_jobs)]
pub(crate) struct NewJobRecord<'a> {
    pub(crate) id: &'a str,
    pub(crate) package_name: &'a str,
    pub(crate) mock_chroot: &'a str,
    pub(crate) revision: &'a str,
    pub(crate) trigger: BuildTrigger,
    pub(crate) status: BuildStatus,
    pub(crate) spec_path: &'a str,
    pub(crate) worker_container_id: Option<&'a str>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) finished_at: Option<String>,
    pub(crate) error_message: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_artifacts)]
pub(crate) struct ArtifactRecord {
    pub(crate) job_id: String,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) arch: String,
    pub(crate) path: String,
    pub(crate) relative_repo_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
}

#[derive(Insertable)]
#[diesel(table_name = build_artifacts)]
pub(crate) struct NewArtifactRecord {
    pub(crate) job_id: String,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) arch: String,
    pub(crate) path: String,
    pub(crate) relative_repo_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
}

#[derive(Insertable)]
#[diesel(table_name = build_logs)]
pub(crate) struct NewBuildLogRecord<'a> {
    pub(crate) job_id: &'a str,
    pub(crate) log_path: &'a str,
    pub(crate) updated_at: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = published_repo_files)]
pub(crate) struct PublishedRepoFileRecord {
    pub(crate) job_id: String,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) arch: String,
    pub(crate) repo_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
    pub(crate) published_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = published_repo_files)]
pub(crate) struct NewPublishedRepoFileRecord {
    pub(crate) job_id: String,
    pub(crate) package_name: String,
    pub(crate) mock_chroot: String,
    pub(crate) arch: String,
    pub(crate) repo_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) kind: ArtifactKind,
    pub(crate) published_at: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct UserRecord {
    pub id: String,
    pub handle: String,
    pub display_name: String,
    pub password_hash: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct UserAuthRecord {
    pub user: UserAccount,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub(crate) struct NewUserRecord<'a> {
    pub(crate) id: &'a str,
    pub(crate) handle: &'a str,
    pub(crate) display_name: &'a str,
    pub(crate) password_hash: &'a str,
    pub(crate) active: bool,
    pub(crate) created_at: &'a str,
    pub(crate) updated_at: &'a str,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = user_permissions)]
pub(crate) struct UserPermissionRecord {
    pub(crate) user_id: String,
    pub(crate) permission: UserPermission,
}

#[derive(Insertable)]
#[diesel(table_name = user_permissions)]
pub(crate) struct NewUserPermissionRecord<'a> {
    pub(crate) user_id: &'a str,
    pub(crate) permission: UserPermission,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = user_repo_metrics)]
pub(crate) struct UserRepoMetricsRecord {
    pub(crate) user_id: String,
    pub(crate) downloaded_bytes: i64,
    pub(crate) updated_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = user_repo_metrics)]
pub(crate) struct NewUserRepoMetricsRecord<'a> {
    pub(crate) user_id: &'a str,
    pub(crate) downloaded_bytes: i64,
    pub(crate) updated_at: &'a str,
}

pub(crate) fn package_response_from_record(
    conn: &mut SqliteConnection,
    record: PackageRecord,
) -> anyhow::Result<PackageResponse> {
    let package = PackageDefinition {
        name: record.name.clone(),
        description: record.description,
        enabled: record.enabled,
        repo_subdir: record.repo_subdir,
        publish_srpm: record.publish_srpm,
        network_access: record.network_access,
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
    Ok(PackageResponse { package, state })
}

pub(crate) fn derive_package_state(
    conn: &mut SqliteConnection,
    package_name: &str,
) -> anyhow::Result<PackageRuntimeState> {
    let last_success = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(build_jobs::status.eq(BuildStatus::Succeeded))
        .order(build_jobs::finished_at.desc())
        .select((build_jobs::id, build_jobs::revision))
        .first::<(String, String)>(conn)
        .optional()?;

    let active_job = build_jobs::table
        .filter(build_jobs::package_name.eq(package_name))
        .filter(
            build_jobs::status
                .eq(BuildStatus::Pending)
                .or(build_jobs::status.eq(BuildStatus::Running)),
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
        active_job_id: active_job.as_deref().map(Uuid::parse_str).transpose()?,
    })
}

pub(crate) fn load_job_responses(
    conn: &mut SqliteConnection,
    rows: Vec<JobRecord>,
) -> anyhow::Result<Vec<BuildJobResponse>> {
    let artifacts = job::load_artifacts_map_for_rows(conn, rows.iter())?;
    rows.into_iter()
        .map(|row| build_job_response_from_row(row, &artifacts))
        .collect()
}

pub(crate) fn build_job_response_from_row(
    row: JobRecord,
    artifacts: &HashMap<Uuid, Vec<BuildArtifact>>,
) -> anyhow::Result<BuildJobResponse> {
    let job = row_to_build_job(row)?;
    Ok(BuildJobResponse {
        artifacts: artifacts.get(&job.id).cloned().unwrap_or_default(),
        job,
    })
}

pub(crate) fn row_to_build_job(row: JobRecord) -> anyhow::Result<BuildJob> {
    Ok(BuildJob {
        id: Uuid::parse_str(&row.id)?,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        revision: row.revision,
        trigger: row.trigger,
        status: row.status,
        spec_path: PathBuf::from(row.spec_path),
        worker_container_id: row.worker_container_id,
        created_at: parse_timestamp(&row.created_at)?,
        updated_at: parse_timestamp(&row.updated_at)?,
        finished_at: row.finished_at.as_deref().map(parse_timestamp).transpose()?,
        error_message: row.error_message,
    })
}

pub(crate) fn parse_timestamp(value: &str) -> anyhow::Result<OffsetDateTime> {
    Ok(OffsetDateTime::parse(value, &Rfc3339)?)
}

pub(crate) fn load_published_repo_files_for_job(
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

pub(crate) fn published_repo_file_from_record(
    row: PublishedRepoFileRecord,
) -> anyhow::Result<PublishedRepoFile> {
    Ok(PublishedRepoFile {
        job_id: Uuid::parse_str(&row.job_id)?,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        arch: row.arch,
        repo_path: PathBuf::from(row.repo_path),
        sha256: row.sha256,
        size_bytes: row.size_bytes.max(0) as u64,
        kind: row.kind,
        published_at: parse_timestamp(&row.published_at)?,
    })
}

pub(crate) fn user_from_record(
    row: UserRecord,
    permissions: Vec<UserPermission>,
) -> anyhow::Result<UserAccount> {
    Ok(UserAccount {
        id: Uuid::parse_str(&row.id)?,
        handle: row.handle,
        display_name: row.display_name,
        active: row.active,
        permissions,
        created_at: parse_timestamp(&row.created_at)?,
        updated_at: parse_timestamp(&row.updated_at)?,
    })
}

pub(crate) fn user_metrics_from_record(row: UserRepoMetricsRecord) -> anyhow::Result<UserRepoMetrics> {
    Ok(UserRepoMetrics {
        user_id: Uuid::parse_str(&row.user_id)?,
        downloaded_bytes: row.downloaded_bytes.max(0) as u64,
        updated_at: parse_timestamp(&row.updated_at)?,
    })
}
