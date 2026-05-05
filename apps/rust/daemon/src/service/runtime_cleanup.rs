use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use tokio::sync::watch;
use tracing::{info, warn};
use uuid::Uuid;

use super::SynforgeService;
use synforge_database::{JobStore, PackageStore};

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
                        if let Err(error) = service.cleanup_orphan_cache_dirs().await {
                            warn!(error = %error, "cache cleanup worker iteration failed");
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
        if worker_jobs_root != jobs_root {
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

        let mut candidates: Vec<(std::path::PathBuf, Uuid)> = Vec::new();
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
            candidates.push((entry.path(), job_id));
        }

        let candidate_ids: Vec<Uuid> = candidates.iter().map(|(_, id)| *id).collect();
        let existing = self.store.filter_existing_job_ids(candidate_ids).await?;

        let mut removed = 0_u64;
        for (path, job_id) in candidates {
            if existing.contains(&job_id) {
                continue;
            }
            tokio::fs::remove_dir_all(path).await?;
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

    /// Sweep ccache and mock-cache roots and remove
    /// `{package}/{chroot}/` subtrees that no longer match a current
    /// package definition. Catches what the explicit hooks can't:
    /// - chroots removed from a package's mock_chroots
    /// - packages deleted / ccache toggled off on the old code path
    /// - any historical dirs that predate this worker
    ///
    /// Policy:
    /// - mock-cache root: orphan if package missing OR chroot missing.
    /// - ccache root: same plus `ccache_enabled = false` reaps the
    ///   whole package tree (matches the immediate cleanup that runs
    ///   on toggle-off via update_package).
    async fn cleanup_orphan_cache_dirs(&self) -> anyhow::Result<()> {
        let snapshot = self.fetch_package_cache_snapshot().await?;
        let ccache_chroots = snapshot.ccache_chroots_by_package();
        let mock_chroots = snapshot.mock_chroots_by_package();

        self.cleanup_orphan_package_chroot_dirs_under(
            &self.config.worker_ccache_root(),
            &ccache_chroots,
            "ccache",
        )
        .await?;
        self.cleanup_orphan_package_chroot_dirs_under(
            &self.config.worker_mock_cache_root(),
            &mock_chroots,
            "mock-cache",
        )
        .await?;
        Ok(())
    }

    async fn fetch_package_cache_snapshot(&self) -> anyhow::Result<PackageCacheSnapshot> {
        const PAGE_SIZE: usize = 200;
        let mut packages: Vec<PackageCacheEntry> = Vec::new();
        let mut offset = 0_usize;
        loop {
            let page = self
                .store
                .list_packages(PAGE_SIZE, offset, None, None)
                .await?;
            if page.is_empty() {
                break;
            }
            offset += page.len();
            for entry in page {
                packages.push(PackageCacheEntry {
                    name: entry.package.name,
                    ccache_enabled: entry.package.ccache_enabled,
                    mock_chroots: entry.package.mock_chroots.into_iter().collect(),
                });
            }
        }
        Ok(PackageCacheSnapshot { packages })
    }

    async fn cleanup_orphan_package_chroot_dirs_under(
        &self,
        root: &Path,
        chroots_by_package: &HashMap<String, HashSet<String>>,
        domain: &'static str,
    ) -> anyhow::Result<()> {
        if !tokio::fs::try_exists(root).await? {
            return Ok(());
        }
        let mut removed_packages = 0_u64;
        let mut removed_chroots = 0_u64;
        let mut entries = tokio::fs::read_dir(root).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let pkg_name = entry.file_name().to_string_lossy().into_owned();
            let pkg_path = entry.path();
            let Some(known_chroots) = chroots_by_package.get(&pkg_name) else {
                tokio::fs::remove_dir_all(&pkg_path).await?;
                removed_packages += 1;
                continue;
            };
            let mut chroot_entries = tokio::fs::read_dir(&pkg_path).await?;
            while let Some(chroot_entry) = chroot_entries.next_entry().await? {
                if !chroot_entry.file_type().await?.is_dir() {
                    continue;
                }
                let chroot = chroot_entry.file_name().to_string_lossy().into_owned();
                if known_chroots.contains(&chroot) {
                    continue;
                }
                tokio::fs::remove_dir_all(chroot_entry.path()).await?;
                removed_chroots += 1;
            }
        }
        if removed_packages > 0 || removed_chroots > 0 {
            info!(
                root = %root.display(),
                domain,
                removed_packages,
                removed_chroots,
                "removed orphan package/chroot cache directories"
            );
        }
        Ok(())
    }
}

struct PackageCacheEntry {
    name: String,
    ccache_enabled: bool,
    mock_chroots: HashSet<String>,
}

struct PackageCacheSnapshot {
    packages: Vec<PackageCacheEntry>,
}

impl PackageCacheSnapshot {
    /// Packages eligible for ccache: only those with `ccache_enabled =
    /// true`. Ones with the flag off act as "no such package" for
    /// the ccache root, so the orphan sweep reaps their cache trees.
    fn ccache_chroots_by_package(&self) -> HashMap<String, HashSet<String>> {
        self.packages
            .iter()
            .filter(|entry| entry.ccache_enabled)
            .map(|entry| (entry.name.clone(), entry.mock_chroots.clone()))
            .collect()
    }

    /// All packages — mock-cache is used regardless of ccache flag.
    fn mock_chroots_by_package(&self) -> HashMap<String, HashSet<String>> {
        self.packages
            .iter()
            .map(|entry| (entry.name.clone(), entry.mock_chroots.clone()))
            .collect()
    }
}
