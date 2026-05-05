mod cache;
mod impls;
mod job;
mod package;
mod records;
mod repo;
mod support;
mod sync;
mod traits;
mod user;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::Context;
use diesel::prelude::*;
use diesel_async::{
    AsyncConnection, AsyncMigrationHarness, AsyncPgConnection, RunQueryDsl,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{Pool, PooledConnection},
    },
};
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
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

use crate::schema::{
    artifact_signatures, build_artifacts, build_failure_backoff, build_jobs, build_logs,
    package_mock_chroots, packages, published_repo_files, runtime_settings, user_permissions,
    user_repo_metrics, users,
};

pub(crate) use records::*;
pub use records::{BuildLogRecord, UserAuthRecord, build_published_repo_path};
pub use traits::{
    BuildFailureBackoffState, GitCacheStore, GitMirrorCacheState, JobStore, PackageStore,
    RepoStore, SyncStore, UserStore,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct DieselStore {
    pool: Pool<AsyncPgConnection>,
}

impl DieselStore {
    pub async fn new(database_url: &str, pool_size: u32) -> anyhow::Result<Self> {
        let connection = AsyncPgConnection::establish(database_url).await?;
        let mut migration_harness = AsyncMigrationHarness::new(connection);
        let applied_migrations = migration_harness
            .run_pending_migrations(MIGRATIONS)
            .map_err(|error| anyhow::anyhow!("failed to run diesel migrations: {}", error))?;
        if applied_migrations.is_empty() {
            info!("no pending diesel migrations");
        } else {
            for migration in applied_migrations {
                info!(migration = %migration, "applied diesel migration");
            }
        }

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder().max_size(pool_size).build(manager).await?;
        Ok(Self { pool })
    }

    pub(crate) async fn get_connection(
        &self,
    ) -> anyhow::Result<PooledConnection<'_, AsyncPgConnection>> {
        self.pool
            .get()
            .await
            .context("failed to acquire diesel connection")
    }
}
