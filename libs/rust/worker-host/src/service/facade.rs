//! Facade for build and worker workflows.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use synforge_core::{
    api::{
        BuildJobResponse, PackageActionDisposition, PackageActionResponse,
        PackageActionTargetResult,
    },
    model::BuildTrigger,
};
use synforge_git_sync::{PackageDefinitionMaterializer, PackageDefinitionWriter};
use tracing::{info, warn};

use super::commands::{
    ActiveTargetBuildReader, BuildJobReader, BuildJobWriter, BuildQueue, ExistingSourceSyncer,
    LastSuccessfulRevisionReader, PackageDefinitionCatalog, PackageDefinitionReader,
    RetryBuildCleaner, RetryJobResetter, TargetBuildBackoffReader, TrackedSourceInspector,
    retry_job, trigger_package_action, trigger_target_action,
};

#[derive(Debug, Clone)]
pub struct BuildService {
    last_polled_at: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Default for BuildService {
    fn default() -> Self {
        Self {
            last_polled_at: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl BuildService {
    pub async fn trigger_package_action<D>(
        &self,
        deps: &D,
        package_name: &str,
        trigger: BuildTrigger,
        force: bool,
    ) -> anyhow::Result<PackageActionResponse>
    where
        D: PackageDefinitionReader
            + TrackedSourceInspector
            + PackageDefinitionMaterializer
            + PackageDefinitionWriter
            + ActiveTargetBuildReader
            + LastSuccessfulRevisionReader
            + TargetBuildBackoffReader
            + BuildJobWriter
            + BuildQueue
            + Send
            + Sync,
    {
        trigger_package_action(deps, package_name, trigger, force).await
    }

    pub async fn trigger_target_action<D>(
        &self,
        deps: &D,
        package_name: &str,
        mock_chroot: &str,
        trigger: BuildTrigger,
        force: bool,
    ) -> anyhow::Result<PackageActionTargetResult>
    where
        D: PackageDefinitionReader
            + TrackedSourceInspector
            + PackageDefinitionMaterializer
            + PackageDefinitionWriter
            + ActiveTargetBuildReader
            + LastSuccessfulRevisionReader
            + TargetBuildBackoffReader
            + BuildJobWriter
            + BuildQueue
            + Send
            + Sync,
    {
        trigger_target_action(deps, package_name, mock_chroot, trigger, force).await
    }

    pub async fn poll_once<D>(&self, deps: &D) -> anyhow::Result<()>
    where
        D: PackageDefinitionCatalog
            + PackageDefinitionReader
            + TrackedSourceInspector
            + PackageDefinitionMaterializer
            + PackageDefinitionWriter
            + ActiveTargetBuildReader
            + LastSuccessfulRevisionReader
            + TargetBuildBackoffReader
            + BuildJobWriter
            + BuildQueue
            + Send
            + Sync,
    {
        for package in deps.list_package_definitions().await? {
            if !package.enabled || !package.source.poll {
                continue;
            }
            if !self.package_is_due(&package.name, package.poll_interval_seconds) {
                continue;
            }
            match trigger_package_action(deps, &package.name, BuildTrigger::Poll, false).await {
                Ok(response) => {
                    let queued_targets = response
                        .results
                        .iter()
                        .filter(|result| result.disposition == PackageActionDisposition::Queued)
                        .count();
                    if queued_targets > 0 {
                        info!(
                            package_name = %package.name,
                            queued_targets,
                            "poll queued build work"
                        );
                    }
                }
                Err(error) => {
                    warn!(
                        package_name = %package.name,
                        error = %error,
                        "poll failed"
                    );
                }
            }
        }
        Ok(())
    }

    pub async fn retry_job<D>(
        &self,
        deps: &D,
        job_id: uuid::Uuid,
    ) -> anyhow::Result<BuildJobResponse>
    where
        D: BuildJobReader
            + PackageDefinitionReader
            + ExistingSourceSyncer
            + ActiveTargetBuildReader
            + RetryBuildCleaner
            + RetryJobResetter
            + BuildQueue
            + Send
            + Sync,
    {
        retry_job(deps, job_id).await
    }

    fn package_is_due(&self, package_name: &str, poll_interval_seconds: u64) -> bool {
        let now = Instant::now();
        let interval = Duration::from_secs(poll_interval_seconds.max(1));
        let mut last_polled_at = self
            .last_polled_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(last_polled_at) = last_polled_at.get(package_name)
            && now.duration_since(*last_polled_at) < interval
        {
            return false;
        }
        last_polled_at.insert(package_name.to_string(), now);
        true
    }
}
