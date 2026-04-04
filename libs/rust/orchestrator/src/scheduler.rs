use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::{DashMap, DashSet};
use thiserror::Error;
use tracing::warn;
use uuid::Uuid;

use synforge_core::{
    api::{BuildJobResponse, PackageActionDisposition, PackageActionTargetResult},
    error::SynforgeError,
    model::{BuildJob, BuildStatus, BuildTrigger, now_utc},
    package::{PackageDefinition, SpecRevision, parse_mock_chroot},
};

use crate::db::{DieselStore, JobStore};
use crate::packages::MaterializePackageOptions;
use crate::registry::PackageRegistry;

#[derive(Clone)]
pub struct QueuedBuild {
    pub package: PackageDefinition,
    pub mock_chroot: String,
    pub revision: SpecRevision,
    pub trigger: BuildTrigger,
    pub job_id: Uuid,
}

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("package {0} is already queued or running")]
    AlreadyQueued(String),
    #[error("no source changes detected")]
    NoSourceChanges,
    #[error("spec Name changed; remove and re-add the package instead")]
    PackageRenamed,
}

struct PackageActionPlan {
    jobs: Vec<BuildJob>,
    queued_builds: Vec<QueuedBuild>,
    #[allow(dead_code)]
    results: Vec<PackageActionTargetResult>,
}

#[derive(Debug, Clone, Eq)]
struct TargetKey {
    package_name: String,
    mock_chroot: String,
}

impl TargetKey {
    fn new(package_name: &str, mock_chroot: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            mock_chroot: mock_chroot.to_string(),
        }
    }
}

impl PartialEq for TargetKey {
    fn eq(&self, other: &Self) -> bool {
        self.package_name == other.package_name && self.mock_chroot == other.mock_chroot
    }
}

impl Hash for TargetKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.package_name.hash(state);
        self.mock_chroot.hash(state);
    }
}

#[derive(Clone)]
pub struct BuildScheduler {
    store: DieselStore,
    registry: PackageRegistry,
    active_targets: Arc<DashSet<TargetKey>>,
    last_polled_at: Arc<DashMap<String, Instant>>,
}

impl BuildScheduler {
    pub fn new(store: DieselStore, registry: PackageRegistry) -> Self {
        Self {
            store,
            registry,
            active_targets: Arc::new(DashSet::new()),
            last_polled_at: Arc::new(DashMap::new()),
        }
    }

    pub async fn poll_once(
        &self,
        queue_tx: &tokio::sync::mpsc::Sender<QueuedBuild>,
    ) -> anyhow::Result<()> {
        for package in self.registry.list_definitions().await? {
            if !package.enabled || !package.source.polling_enabled() {
                continue;
            }
            if !self.package_is_due(&package.name, package.poll_interval_seconds) {
                continue;
            }
            if let Err(error) = self
                .enqueue_package(&package.name, BuildTrigger::Poll, false, queue_tx)
                .await
            {
                if matches!(
                    error.downcast_ref::<SchedulerError>(),
                    Some(SchedulerError::AlreadyQueued(_) | SchedulerError::NoSourceChanges)
                ) {
                    continue;
                }
                warn!("poll failed for {}: {}", package.name, error);
            }
        }
        Ok(())
    }

    fn package_is_due(&self, package_name: &str, poll_interval_seconds: u64) -> bool {
        let now = Instant::now();
        let interval = Duration::from_secs(poll_interval_seconds.max(1));
        if let Some(last_polled_at) = self.last_polled_at.get(package_name)
            && now.duration_since(*last_polled_at) < interval
        {
            return false;
        }
        self.last_polled_at.insert(package_name.to_string(), now);
        true
    }

    pub async fn enqueue_package(
        &self,
        package_name: &str,
        trigger: BuildTrigger,
        force: bool,
        queue_tx: &tokio::sync::mpsc::Sender<QueuedBuild>,
    ) -> anyhow::Result<BuildJobResponse> {
        let package = self.registry.get_definition(package_name).await?;

        let result = self.prepare_package_action(package, trigger, force).await;

        match result {
            Ok(plan) => {
                let jobs = plan.jobs;
                let queued_builds = plan.queued_builds;
                let response_job_id = jobs
                    .first()
                    .map(|job| job.id)
                    .ok_or_else(|| anyhow::anyhow!("no build jobs were created"))?;

                // Mark all targets as active before inserting jobs
                let mut reserved_targets = Vec::new();
                for job in &jobs {
                    let key = TargetKey::new(&job.package_name, &job.mock_chroot);
                    self.active_targets.insert(key.clone());
                    reserved_targets.push(key);
                }

                for job in &jobs {
                    self.store.insert_job(job).await?;
                }
                for queued in queued_builds {
                    if let Err(error) = queue_tx.send(queued).await {
                        // Rollback: release all reserved targets
                        for key in &reserved_targets {
                            self.active_targets.remove(key);
                        }
                        return Err(anyhow::anyhow!("failed to queue build: {}", error));
                    }
                }
                self.store.get_job(response_job_id).await?.ok_or_else(|| {
                    anyhow::anyhow!(SynforgeError::NotFound(response_job_id.to_string()))
                })
            }
            Err(error) => Err(error),
        }
    }

    pub fn release_target(&self, package_name: &str, mock_chroot: &str) {
        self.active_targets
            .remove(&TargetKey::new(package_name, mock_chroot));
    }

    fn target_is_active(&self, package_name: &str, mock_chroot: &str) -> bool {
        self.active_targets
            .contains(&TargetKey::new(package_name, mock_chroot))
    }

    async fn prepare_package_action(
        &self,
        package: PackageDefinition,
        trigger: BuildTrigger,
        force: bool,
    ) -> anyhow::Result<PackageActionPlan> {
        let inspected = self
            .registry
            .inspect_source(
                &package.name,
                &package.source,
                package.build_timeout_seconds,
            )
            .await?;
        let revision_key = inspected.revision.comparison_key();

        let build_chroots = package.mock_chroots.clone();
        let mut queued_chroots = Vec::new();
        let mut blocked_by_active_job = false;
        let mut results = Vec::new();
        for mock_chroot in &build_chroots {
            // Check both in-memory (queued but not yet in DB) and DB (running)
            if self.target_is_active(&package.name, mock_chroot)
                || self
                    .store
                    .has_active_job_for_target(&package.name, mock_chroot)
                    .await?
            {
                blocked_by_active_job = true;
                results.push(PackageActionTargetResult {
                    package_name: package.name.clone(),
                    mock_chroot: mock_chroot.clone(),
                    disposition: PackageActionDisposition::Blocked,
                    reason: Some("pending_or_running".to_string()),
                    job_id: None,
                    revision: Some(revision_key.clone()),
                });
                continue;
            }
            let previous_revision = self
                .store
                .get_last_successful_revision(&package.name, mock_chroot)
                .await?;
            if force || previous_revision.as_deref() != Some(revision_key.as_str()) {
                queued_chroots.push(mock_chroot.clone());
            } else {
                results.push(PackageActionTargetResult {
                    package_name: package.name.clone(),
                    mock_chroot: mock_chroot.clone(),
                    disposition: PackageActionDisposition::Skipped,
                    reason: Some("no_source_change".to_string()),
                    job_id: None,
                    revision: Some(revision_key.clone()),
                });
            }
        }
        if queued_chroots.is_empty() {
            if blocked_by_active_job {
                return Err(SchedulerError::AlreadyQueued(package.name.clone()).into());
            }
            return Err(SchedulerError::NoSourceChanges.into());
        }

        let updated_package = self
            .registry
            .materialize_inspected_source(
                &package.source,
                &inspected,
                MaterializePackageOptions {
                    enabled: package.enabled,
                    network_access: package.network_access,
                    mock_chroots: package.mock_chroots.clone(),
                    poll_interval_seconds: package.poll_interval_seconds,
                    build_timeout_seconds: package.build_timeout_seconds,
                    package_history_count: package.package_history_count,
                    build_env: package.build_env.clone(),
                },
            )
            .await?;
        if updated_package.name != package.name {
            return Err(SchedulerError::PackageRenamed.into());
        }
        self.store.upsert_package(&updated_package).await?;

        let now = now_utc();
        let mut jobs = Vec::new();
        let mut queued = Vec::new();
        for mock_chroot in queued_chroots {
            parse_mock_chroot(&mock_chroot)
                .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", mock_chroot))?;
            let job_id = Uuid::now_v7();
            let job = BuildJob {
                id: job_id,
                package_name: updated_package.name.clone(),
                mock_chroot: mock_chroot.clone(),
                revision: revision_key.clone(),
                trigger,
                status: BuildStatus::Pending,
                spec_path: updated_package.spec_path.clone(),
                worker_container_id: None,
                created_at: now,
                updated_at: now,
                finished_at: None,
                error_message: None,
            };
            queued.push(QueuedBuild {
                package: updated_package.clone(),
                mock_chroot,
                revision: inspected.revision.clone(),
                trigger,
                job_id,
            });
            results.push(PackageActionTargetResult {
                package_name: updated_package.name.clone(),
                mock_chroot: job.mock_chroot.clone(),
                disposition: PackageActionDisposition::Queued,
                reason: None,
                job_id: Some(job_id),
                revision: Some(revision_key.clone()),
            });
            jobs.push(job);
        }
        Ok(PackageActionPlan {
            jobs,
            queued_builds: queued,
            results,
        })
    }
}
