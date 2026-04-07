use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::watch;
use tracing::{info, warn};
use uuid::Uuid;

use super::SynforgeService;
use crate::db::JobStore;

const JOB_RUNTIME_CLEANUP_TICK_SECONDS: u64 = 120;

impl SynforgeService {
    pub(super) fn start_runtime_cleanup_worker(
        self: &Arc<Self>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let service = Arc::clone(self);
        info!(
            cleanup_tick_seconds = JOB_RUNTIME_CLEANUP_TICK_SECONDS,
            "starting runtime cleanup worker"
        );
        self.task_tracker.spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(
                JOB_RUNTIME_CLEANUP_TICK_SECONDS,
            ));
            // Skip the immediate first tick to avoid running cleanup during startup.
            ticker.tick().await;
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    _ = ticker.tick() => {
                        if let Err(error) = service.cleanup_orphan_job_runtime_dirs().await {
                            warn!(error = %error, "runtime cleanup worker iteration failed");
                        }
                    }
                }
            }
        });
    }

    async fn cleanup_orphan_job_runtime_dirs(&self) -> anyhow::Result<()> {
        let active_job_ids = self
            .sessions
            .active_job_ids()
            .into_iter()
            .collect::<HashSet<Uuid>>();
        let jobs_root = self.config.runtime_paths().jobs_root().to_path_buf();
        self.cleanup_orphan_job_dirs_under(&jobs_root, &active_job_ids, "runtime")
            .await?;

        let worker_jobs_root = self.config.worker_jobs_root();
        if worker_jobs_root.as_path() != jobs_root.as_path() {
            self.cleanup_orphan_job_dirs_under(&worker_jobs_root, &active_job_ids, "worker")
                .await?;
        }
        Ok(())
    }

    async fn cleanup_orphan_job_dirs_under(
        &self,
        root: &Path,
        active_job_ids: &HashSet<Uuid>,
        domain: &'static str,
    ) -> anyhow::Result<()> {
        if !tokio::fs::try_exists(root).await? {
            return Ok(());
        }

        let mut removed = 0_u64;
        let mut scanned = 0_u64;
        let mut entries = tokio::fs::read_dir(root).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Ok(job_id) = Uuid::parse_str(file_name) else {
                continue;
            };
            scanned += 1;
            if active_job_ids.contains(&job_id) {
                continue;
            }
            if self.store.get_job(job_id).await?.is_some() {
                continue;
            }
            tokio::fs::remove_dir_all(entry.path()).await?;
            removed += 1;
        }

        if removed > 0 {
            info!(
                root = %root.display(),
                domain,
                scanned,
                removed,
                "removed orphan job directories"
            );
        }
        Ok(())
    }
}
