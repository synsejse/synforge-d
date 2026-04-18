mod browse;
mod git;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use synforge_core::{
    api::BrowseRepositoryProgressView,
    config::DaemonConfig,
    constants::DEFAULT_WORKER_WORKSPACE_ROOT,
    model::{WorkerAction, WorkerJobPayload, WorkerParsePayload, WorkerResult},
    package::{BuildEnvVar, PackageDefinition, SpecRevision, SpecSource},
    validation::{PackageDefinitionValidator, Validator},
};
use synforge_database::DieselStore;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::cache::{GitMirrorCache, GitMirrorCacheStatsSnapshot};

#[derive(Debug, Clone)]
pub struct InspectedPackageSource {
    pub package_name: String,
    pub description: String,
    pub revision: SpecRevision,
}

#[derive(Debug, Clone)]
pub struct MaterializePackageOptions {
    pub enabled: bool,
    pub publish_srpm: bool,
    pub publish_debuginfo: bool,
    pub network_access: bool,
    pub ccache_enabled: bool,
    pub ccache_max_size_mb: Option<u64>,
    pub mock_chroots: Vec<String>,
    pub poll_interval_seconds: u64,
    pub build_timeout_seconds: u64,
    pub package_history_count: u64,
    pub cpu_limit_millicores: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub build_env: Vec<BuildEnvVar>,
}

#[async_trait]
pub trait WorkerParseRunner: Send + Sync {
    async fn run_job(
        &self,
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<WorkerResult>;
}

#[derive(Clone)]
pub struct PackageSyncStore {
    config: DaemonConfig,
    worker_runner: Arc<dyn WorkerParseRunner>,
    git_mirror_cache: Arc<GitMirrorCache>,
    browse_progress: Arc<Mutex<Option<BrowseRepositoryProgressView>>>,
}

impl PackageSyncStore {
    pub fn new(
        config: DaemonConfig,
        worker_runner: Arc<dyn WorkerParseRunner>,
        store: DieselStore,
    ) -> Self {
        let git_mirror_cache = Arc::new(GitMirrorCache::new(store, &config));
        Self {
            config,
            worker_runner,
            git_mirror_cache,
            browse_progress: Arc::new(Mutex::new(None)),
        }
    }

    pub fn git_mirror_root(&self) -> PathBuf {
        self.git_mirror_cache.mirror_root().to_path_buf()
    }

    pub async fn cleanup_git_mirror_cache(&self) -> anyhow::Result<()> {
        let summary = self.git_mirror_cache.cleanup_stale().await?;
        self.git_mirror_cache.log_cleanup_result(summary);
        Ok(())
    }

    pub async fn git_cache_stats(&self) -> anyhow::Result<GitMirrorCacheStatsSnapshot> {
        self.git_mirror_cache.stats().await
    }

    pub async fn browse_repository_progress(&self) -> Option<BrowseRepositoryProgressView> {
        self.browse_progress.lock().await.clone()
    }

    #[instrument(skip(self, source), fields(package_name = %package_name, repo_url = %source.repo_url))]
    pub async fn inspect_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
    ) -> anyhow::Result<InspectedPackageSource> {
        if let Err(error) = self.git_mirror_cache.ensure_mirror(&source.repo_url).await {
            tracing::warn!(
                package_name,
                repo_url = %source.repo_url,
                error = %error,
                "failed to warm git mirror cache before inspect; continuing with direct clone"
            );
        }
        let job_id = uuid::Uuid::now_v7();
        let workspace_dir = Path::new(DEFAULT_WORKER_WORKSPACE_ROOT).join(job_id.to_string());
        let payload = WorkerJobPayload {
            job_id,
            workspace_dir,
            timeout_seconds,
            action: WorkerAction::Parse(WorkerParsePayload {
                package_name: package_name.to_string(),
                source: source.clone(),
            }),
        };
        let execution = self.worker_runner.run_job(&payload, &self.config).await?;
        let WorkerResult::Parse(result) = execution else {
            return Err(anyhow::anyhow!("worker did not return a parse result"));
        };

        Ok(InspectedPackageSource {
            package_name: package_name.to_string(),
            description: result.parsed.summary,
            revision: result.revision,
        })
    }

    pub async fn sync_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        options: MaterializePackageOptions,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)> {
        let inspected = self
            .inspect_source(package_name, source, options.build_timeout_seconds)
            .await?;
        let package = self.materialize(source, &inspected, options).await?;
        Ok((package, inspected.revision.clone()))
    }

    pub async fn materialize(
        &self,
        source: &SpecSource,
        inspected: &InspectedPackageSource,
        options: MaterializePackageOptions,
    ) -> anyhow::Result<PackageDefinition> {
        let package = PackageDefinition {
            name: inspected.package_name.clone(),
            description: inspected.description.clone(),
            enabled: options.enabled,
            repo_subdir: inspected.package_name.clone(),
            publish_srpm: options.publish_srpm,
            publish_debuginfo: options.publish_debuginfo,
            network_access: options.network_access,
            ccache_enabled: options.ccache_enabled,
            ccache_max_size_mb: options.ccache_max_size_mb,
            mock_chroots: options.mock_chroots,
            source: source.clone(),
            poll_interval_seconds: options.poll_interval_seconds,
            build_timeout_seconds: options.build_timeout_seconds,
            package_history_count: options.package_history_count,
            cpu_limit_millicores: options.cpu_limit_millicores,
            memory_limit_mb: options.memory_limit_mb,
            build_env: options.build_env,
            spec_file: PathBuf::from(&source.spec_file),
            version: inspected.revision.version.clone(),
            release: inspected.revision.release.clone(),
        };
        PackageDefinitionValidator.validate(&package)?;
        Ok(package)
    }

    pub async fn delete(&self, package_name: &str) -> anyhow::Result<()> {
        let _ = package_name;
        Ok(())
    }
}
