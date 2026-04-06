mod cache;
mod job;
mod package;
mod repo;
mod sync;
mod traits;
mod user;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use synforge_core::{
    api::{BuildJobResponse, PackageResponse, RepoTargetSummary},
    model::{
        ArtifactKind, ArtifactSignature, ArtifactSigningStatus, BuildArtifact, BuildJob,
        BuildStatus, BuildTrigger, PackageRuntimeState, PackageTargetRuntimeState,
        PublishedRepoFile, UserAccount, UserPermission, UserRepoMetrics, UserSummary,
        format_timestamp, now_utc,
    },
    package::{BuildEnvVar, PackageDefinition, SpecSource},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::info;
use uuid::Uuid;

use crate::schema::{
    artifact_signatures, build_artifacts, build_jobs, build_logs, packages, published_repo_files,
    runtime_settings, user_permissions, user_repo_metrics, users,
};

pub use traits::{
    GitCacheStore, GitMirrorCacheState, JobStore, PackageStore, RepoStore, SyncStore, UserStore,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct DieselStore {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl DieselStore {
    pub async fn new(database_url: &str, pool_size: u32) -> anyhow::Result<Self> {
        let database_url = database_url.to_string();
        let pool = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let manager = ConnectionManager::<MysqlConnection>::new(&database_url);
            let pool = Pool::builder().max_size(pool_size).build(manager)?;
            let mut conn = pool.get()?;
            let applied_migrations = conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!("failed to run diesel migrations: {}", error))?;
            if applied_migrations.is_empty() {
                info!("no pending diesel migrations");
            } else {
                for migration in applied_migrations {
                    info!(migration = %migration, "applied diesel migration");
                }
            }
            Ok(pool)
        })
        .await
        .context("diesel store initialization task failed")??;
        Ok(Self { pool })
    }

    pub(crate) async fn with_connection<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut MysqlConnection) -> anyhow::Result<T> + Send + 'static,
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

    pub async fn list_runtime_settings(
        &self,
    ) -> anyhow::Result<BTreeMap<String, serde_json::Value>> {
        self.with_connection(|conn| {
            let rows = runtime_settings::table
                .select((runtime_settings::key, runtime_settings::value_json))
                .load::<(String, String)>(conn)?;
            let mut settings = BTreeMap::new();
            for (key, value_json) in rows {
                let value = serde_json::from_str::<serde_json::Value>(&value_json)
                    .with_context(|| format!("invalid runtime setting JSON for key {}", key))?;
                settings.insert(key, value);
            }
            Ok(settings)
        })
        .await
    }

    pub async fn upsert_runtime_settings(
        &self,
        settings: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<()> {
        self.with_connection(move |conn| {
            let updated_at = format_timestamp(now_utc());
            for (key, value) in settings {
                let value_json = serde_json::to_string(&value)
                    .with_context(|| format!("failed to serialize runtime setting {}", key))?;
                let existing = runtime_settings::table
                    .find(key.as_str())
                    .select(runtime_settings::key)
                    .first::<String>(conn)
                    .optional()?;
                if existing.is_some() {
                    diesel::update(runtime_settings::table.find(key.as_str()))
                        .set((
                            runtime_settings::value_json.eq(value_json.as_str()),
                            runtime_settings::updated_at.eq(updated_at.as_str()),
                        ))
                        .execute(conn)?;
                } else {
                    let row = NewRuntimeSettingRecord {
                        key: key.as_str(),
                        value_json: value_json.as_str(),
                        updated_at: updated_at.as_str(),
                    };
                    diesel::insert_into(runtime_settings::table)
                        .values(&row)
                        .execute(conn)?;
                }
            }
            Ok(())
        })
        .await
    }

    pub async fn update_build_artifact_metadata(
        &self,
        artifact_id: Uuid,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<()> {
        let artifact_id = artifact_id.to_string();
        self.with_connection(move |conn| {
            diesel::update(build_artifacts::table.find(artifact_id.as_str()))
                .set((
                    build_artifacts::sha256.eq(sha256.as_str()),
                    build_artifacts::size_bytes.eq(size_bytes as i64),
                ))
                .execute(conn)?;
            Ok(())
        })
        .await
    }

    pub async fn upsert_artifact_signatures(
        &self,
        signatures: Vec<ArtifactSignature>,
    ) -> anyhow::Result<()> {
        self.with_connection(move |conn| {
            let updated_at = format_timestamp(now_utc());
            for signature in signatures {
                let artifact_id = signature.artifact_id.to_string();
                let existing = artifact_signatures::table
                    .find(artifact_id.as_str())
                    .select(artifact_signatures::artifact_id)
                    .first::<String>(conn)
                    .optional()?;
                if existing.is_some() {
                    diesel::update(artifact_signatures::table.find(artifact_id.as_str()))
                        .set((
                            artifact_signatures::status.eq(signature.status),
                            artifact_signatures::signed_at
                                .eq(signature.signed_at.map(format_timestamp)),
                            artifact_signatures::key_id.eq(signature.key_id.as_deref()),
                            artifact_signatures::fingerprint.eq(signature.fingerprint.as_deref()),
                            artifact_signatures::error_message
                                .eq(signature.error_message.as_deref()),
                            artifact_signatures::updated_at.eq(updated_at.as_str()),
                        ))
                        .execute(conn)?;
                } else {
                    let row = NewArtifactSignatureRecord {
                        artifact_id,
                        status: signature.status,
                        signed_at: signature.signed_at.map(format_timestamp),
                        key_id: signature.key_id,
                        fingerprint: signature.fingerprint,
                        error_message: signature.error_message,
                        updated_at: updated_at.clone(),
                    };
                    diesel::insert_into(artifact_signatures::table)
                        .values(&row)
                        .execute(conn)?;
                }
            }
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl PackageStore for DieselStore {
    async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>> {
        package::list_packages(self, limit, offset, search, enabled).await
    }

    async fn count_packages(
        &self,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<u64> {
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
}

#[async_trait]
impl JobStore for DieselStore {
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

    async fn set_job_running(
        &self,
        job_id: Uuid,
        worker_container_id: Option<&str>,
    ) -> anyhow::Result<()> {
        job::set_job_running(self, job_id, worker_container_id).await
    }

    async fn finish_job(
        &self,
        job_id: Uuid,
        status: BuildStatus,
        error_message: Option<&str>,
        artifacts: &[BuildArtifact],
        published_files: &[PublishedRepoFile],
        artifact_signatures: &[ArtifactSignature],
    ) -> anyhow::Result<()> {
        job::finish_job(
            self,
            job_id,
            status,
            error_message,
            artifacts,
            published_files,
            artifact_signatures,
        )
        .await
    }

    async fn list_build_logs_for_job(&self, job_id: Uuid) -> anyhow::Result<Vec<BuildLogRecord>> {
        job::list_build_logs_for_job(self, job_id).await
    }

    async fn upsert_build_log(&self, job_id: Uuid, file: &str) -> anyhow::Result<()> {
        job::upsert_build_log(self, job_id, file).await
    }

    async fn get_build_log_for_job_source(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<Option<BuildLogRecord>> {
        job::get_build_log_for_job_source(self, job_id, file).await
    }

    async fn list_jobs(
        &self,
        limit: usize,
        offset: usize,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs(
            self,
            limit,
            offset,
            status,
            package_name,
            mock_chroot,
            completed_only,
        )
        .await
    }

    async fn count_jobs(
        &self,
        status: Option<BuildStatus>,
        package_name: Option<String>,
        mock_chroot: Option<String>,
        completed_only: bool,
    ) -> anyhow::Result<u64> {
        job::count_jobs(self, status, package_name, mock_chroot, completed_only).await
    }

    async fn list_active_jobs(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_active_jobs(self, limit, offset, package_name, mock_chroot).await
    }

    async fn count_active_jobs(
        &self,
        package_name: Option<String>,
        mock_chroot: Option<String>,
    ) -> anyhow::Result<u64> {
        job::count_active_jobs(self, package_name, mock_chroot).await
    }

    async fn list_jobs_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        job::list_jobs_for_package(self, package_name).await
    }

    async fn get_job(&self, job_id: Uuid) -> anyhow::Result<Option<BuildJobResponse>> {
        job::get_job(self, job_id).await
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
}

#[async_trait]
impl RepoStore for DieselStore {
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

    async fn list_published_repo_files_for_job(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_job(self, job_id).await
    }

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        repo::list_published_repo_files_for_package(self, package_name).await
    }

    async fn list_recent_published_repo_files(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
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
}

#[async_trait]
impl UserStore for DieselStore {
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

    async fn get_user_auth_by_handle(
        &self,
        handle: &str,
    ) -> anyhow::Result<Option<UserAuthRecord>> {
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
        user::create_user(
            self,
            handle,
            display_name,
            password_hash,
            active,
            permissions,
        )
        .await
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

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> anyhow::Result<bool> {
        user::update_user_password(self, user_id, password_hash).await
    }

    async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<Option<UserSummary>> {
        user::delete_user(self, user_id).await
    }

    async fn increment_user_download_bytes(&self, user_id: Uuid, bytes: u64) -> anyhow::Result<()> {
        user::increment_user_download_bytes(self, user_id, bytes).await
    }
}

#[async_trait]
impl SyncStore for DieselStore {
    async fn insert_sync_operation(
        &self,
        package_name: &str,
        trigger_type: synforge_core::sync::SyncTriggerType,
        status: synforge_core::sync::SyncStatus,
        revision: Option<&str>,
        error_message: Option<&str>,
    ) -> anyhow::Result<()> {
        sync::insert_sync_operation(
            self,
            package_name,
            trigger_type,
            status,
            revision,
            error_message,
        )
        .await
    }

    async fn list_sync_operations(
        &self,
        limit: usize,
        offset: usize,
        package_name: Option<String>,
        status: Option<synforge_core::sync::SyncStatus>,
    ) -> anyhow::Result<Vec<synforge_core::sync::SyncOperation>> {
        sync::list_sync_operations(self, limit, offset, package_name, status).await
    }

    async fn count_sync_operations(
        &self,
        package_name: Option<String>,
        status: Option<synforge_core::sync::SyncStatus>,
    ) -> anyhow::Result<u64> {
        sync::count_sync_operations(self, package_name, status).await
    }

    async fn get_sync_metrics(&self) -> anyhow::Result<(usize, usize, Option<String>)> {
        sync::get_sync_metrics(self).await
    }
}

#[async_trait]
impl GitCacheStore for DieselStore {
    async fn get_git_mirror_cache_state(
        &self,
        mirror_key: &str,
    ) -> anyhow::Result<Option<GitMirrorCacheState>> {
        cache::get_git_mirror_cache_state(self, mirror_key).await
    }

    async fn upsert_git_mirror_cache_state(
        &self,
        mirror_key: &str,
        repo_url: &str,
        last_fetched_at: i64,
        last_used_at: i64,
    ) -> anyhow::Result<()> {
        cache::upsert_git_mirror_cache_state(
            self,
            mirror_key,
            repo_url,
            last_fetched_at,
            last_used_at,
        )
        .await
    }

    async fn list_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<Vec<GitMirrorCacheState>> {
        cache::list_stale_git_mirror_cache_states(self, last_used_before_or_equal).await
    }

    async fn delete_git_mirror_cache_state(&self, mirror_key: &str) -> anyhow::Result<()> {
        cache::delete_git_mirror_cache_state(self, mirror_key).await
    }

    async fn count_git_mirror_cache_states(&self) -> anyhow::Result<u64> {
        cache::count_git_mirror_cache_states(self).await
    }

    async fn count_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<u64> {
        cache::count_stale_git_mirror_cache_states(self, last_used_before_or_equal).await
    }

    async fn latest_git_mirror_cache_timestamps(
        &self,
    ) -> anyhow::Result<(Option<i64>, Option<i64>)> {
        cache::latest_git_mirror_cache_timestamps(self).await
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
    pub(crate) publish_debuginfo: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: String,
    pub(crate) source_repo_url: String,
    pub(crate) source_spec_file: String,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) build_env_json: String,
    pub(crate) spec_file: String,
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
    pub(crate) publish_debuginfo: bool,
    pub(crate) network_access: bool,
    pub(crate) mock_chroots_json: &'a str,
    pub(crate) source_repo_url: &'a str,
    pub(crate) source_spec_file: &'a str,
    pub(crate) source_poll: bool,
    pub(crate) poll_interval_seconds: i64,
    pub(crate) build_timeout_seconds: i64,
    pub(crate) package_history_count: i64,
    pub(crate) build_env_json: &'a str,
    pub(crate) spec_file: &'a str,
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
    pub(crate) spec_file: String,
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
    pub(crate) spec_file: &'a str,
    pub(crate) worker_container_id: Option<&'a str>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) finished_at: Option<String>,
    pub(crate) error_message: Option<&'a str>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = build_artifacts)]
pub(crate) struct ArtifactRecord {
    pub(crate) id: String,
    pub(crate) job_id: String,
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
    pub(crate) id: String,
    pub(crate) job_id: String,
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
    pub(crate) artifact_id: String,
    pub(crate) status: ArtifactSigningStatus,
    pub(crate) signed_at: Option<String>,
    pub(crate) key_id: Option<String>,
    pub(crate) fingerprint: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) updated_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = build_logs)]
pub(crate) struct NewBuildLogRecord<'a> {
    pub(crate) job_id: &'a str,
    pub(crate) file: &'a str,
    pub(crate) updated_at: &'a str,
}

#[derive(Debug, Queryable)]
pub struct BuildLogRecord {
    pub file: String,
}

#[derive(Insertable)]
#[diesel(table_name = published_repo_files)]
pub(crate) struct NewPublishedRepoFileRecord {
    pub(crate) artifact_id: String,
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

#[derive(Insertable)]
#[diesel(table_name = runtime_settings)]
pub(crate) struct NewRuntimeSettingRecord<'a> {
    pub(crate) key: &'a str,
    pub(crate) value_json: &'a str,
    pub(crate) updated_at: &'a str,
}

pub(crate) fn package_response_from_record(
    conn: &mut MysqlConnection,
    record: PackageRecord,
) -> anyhow::Result<PackageResponse> {
    let package = PackageDefinition {
        name: record.name.clone(),
        description: record.description,
        enabled: record.enabled,
        repo_subdir: record.repo_subdir,
        publish_srpm: record.publish_srpm,
        publish_debuginfo: record.publish_debuginfo,
        network_access: record.network_access,
        mock_chroots: serde_json::from_str::<Vec<String>>(&record.mock_chroots_json)
            .unwrap_or_default(),
        source: SpecSource {
            repo_url: record.source_repo_url,
            spec_file: record.source_spec_file,
            poll: record.source_poll,
        },
        poll_interval_seconds: record.poll_interval_seconds as u64,
        build_timeout_seconds: record.build_timeout_seconds as u64,
        package_history_count: record.package_history_count as u64,
        build_env: serde_json::from_str::<Vec<BuildEnvVar>>(&record.build_env_json)
            .unwrap_or_default(),
        spec_file: PathBuf::from(record.spec_file),
        version: record.version,
        release: record.release,
    };
    let state = compute_package_state(conn, &record.name, &package.mock_chroots)?;
    Ok(PackageResponse { package, state })
}

pub(crate) fn compute_package_state(
    conn: &mut MysqlConnection,
    package_name: &str,
    mock_chroots: &[String],
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

    let mut targets = Vec::with_capacity(mock_chroots.len());
    for mock_chroot in mock_chroots {
        let last_success = build_jobs::table
            .filter(build_jobs::package_name.eq(package_name))
            .filter(build_jobs::mock_chroot.eq(mock_chroot))
            .filter(build_jobs::status.eq(BuildStatus::Succeeded))
            .order(build_jobs::finished_at.desc())
            .select((build_jobs::id, build_jobs::revision))
            .first::<(String, String)>(conn)
            .optional()?;

        let active_job = build_jobs::table
            .filter(build_jobs::package_name.eq(package_name))
            .filter(build_jobs::mock_chroot.eq(mock_chroot))
            .filter(
                build_jobs::status
                    .eq(BuildStatus::Pending)
                    .or(build_jobs::status.eq(BuildStatus::Running)),
            )
            .order(build_jobs::created_at.desc())
            .select((build_jobs::id, build_jobs::status))
            .first::<(String, BuildStatus)>(conn)
            .optional()?;

        targets.push(PackageTargetRuntimeState {
            mock_chroot: mock_chroot.clone(),
            last_revision: last_success.as_ref().map(|(_, revision)| revision.clone()),
            last_successful_build_id: last_success
                .as_ref()
                .map(|(id, _)| Uuid::parse_str(id))
                .transpose()?,
            active_job_id: active_job
                .as_ref()
                .map(|(id, _)| Uuid::parse_str(id))
                .transpose()?,
            active_status: active_job.as_ref().map(|(_, status)| *status),
        });
    }

    Ok(PackageRuntimeState {
        last_revision: last_success.as_ref().map(|(_, revision)| revision.clone()),
        last_successful_build_id: last_success
            .as_ref()
            .map(|(id, _)| Uuid::parse_str(id))
            .transpose()?,
        active_job_id: active_job.as_deref().map(Uuid::parse_str).transpose()?,
        targets,
    })
}

pub(crate) fn load_job_responses(
    conn: &mut MysqlConnection,
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
    let job = job_from_row(row)?;
    Ok(BuildJobResponse {
        artifacts: artifacts.get(&job.id).cloned().unwrap_or_default(),
        job,
    })
}

pub(crate) fn job_from_row(row: JobRecord) -> anyhow::Result<BuildJob> {
    Ok(BuildJob {
        id: Uuid::parse_str(&row.id)?,
        package_name: row.package_name,
        mock_chroot: row.mock_chroot,
        revision: row.revision,
        trigger: row.trigger,
        status: row.status,
        spec_file: PathBuf::from(row.spec_file),
        worker_container_id: row.worker_container_id,
        created_at: parse_timestamp(&row.created_at)?,
        updated_at: parse_timestamp(&row.updated_at)?,
        finished_at: row
            .finished_at
            .as_deref()
            .map(parse_timestamp)
            .transpose()?,
        error_message: row.error_message,
    })
}

pub(crate) fn parse_timestamp(value: &str) -> anyhow::Result<OffsetDateTime> {
    Ok(OffsetDateTime::parse(value, &Rfc3339)?)
}

pub(crate) fn load_published_repo_files_for_job(
    conn: &mut MysqlConnection,
    job_id: &str,
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
        .load::<(
            String,
            String,
            String,
            String,
            String,
            String,
            i64,
            ArtifactKind,
            String,
            Option<ArtifactSigningStatus>,
            Option<String>,
        )>(conn)?;
    rows.into_iter()
        .map(published_repo_file_from_record)
        .collect()
}

pub(crate) fn published_repo_file_from_record(
    row: (
        String,
        String,
        String,
        String,
        String,
        String,
        i64,
        ArtifactKind,
        String,
        Option<ArtifactSigningStatus>,
        Option<String>,
    ),
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
    let job_id = Uuid::parse_str(&job_id)?;
    let path = build_published_repo_path(
        &package_name,
        &mock_chroot,
        job_id,
        Path::new(&artifact_path),
    )?;
    Ok(PublishedRepoFile {
        artifact_id: Uuid::parse_str(&artifact_id)?,
        job_id,
        package_name,
        mock_chroot,
        path,
        sha256,
        size_bytes: size_bytes.max(0) as u64,
        kind,
        published_at: parse_timestamp(&published_at)?,
        signing_status,
        signing_error_message,
    })
}

pub(crate) fn build_published_repo_path(
    package_name: &str,
    mock_chroot: &str,
    job_id: Uuid,
    artifact_path: &Path,
) -> anyhow::Result<PathBuf> {
    let file_name = artifact_path.file_name().ok_or_else(|| {
        anyhow::anyhow!(
            "artifact path {} has no filename for published repo path",
            artifact_path.display()
        )
    })?;
    Ok(PathBuf::from("packages")
        .join(package_name)
        .join(mock_chroot)
        .join("builds")
        .join(job_id.to_string())
        .join(file_name))
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

pub(crate) fn user_metrics_from_record(
    row: UserRepoMetricsRecord,
) -> anyhow::Result<UserRepoMetrics> {
    Ok(UserRepoMetrics {
        user_id: Uuid::parse_str(&row.user_id)?,
        downloaded_bytes: row.downloaded_bytes.max(0) as u64,
        updated_at: parse_timestamp(&row.updated_at)?,
    })
}
