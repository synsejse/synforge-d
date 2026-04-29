use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use dashmap::DashMap;
use sha2::{Digest, Sha256};
use synforge_core::config::DaemonConfig;
use synforge_database::{DieselStore, GitCacheStore};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Clone)]
pub(crate) struct GitMirrorCache {
    store: DieselStore,
    root: PathBuf,
    refresh_ttl: Duration,
    max_unused: Duration,
    git_timeout: Duration,
    repo_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct GitMirrorCleanupSummary {
    pub scanned: usize,
    pub removed: usize,
}

#[derive(Debug, Clone)]
pub struct GitMirrorCacheStatsSnapshot {
    pub mirror_root: PathBuf,
    pub refresh_ttl_seconds: u64,
    pub max_unused_seconds: u64,
    pub tracked_mirrors: u64,
    pub stale_mirrors: u64,
    pub mirror_directories: usize,
    pub latest_fetched_at: Option<i64>,
    pub latest_used_at: Option<i64>,
}

impl GitMirrorCache {
    pub(crate) fn new(store: DieselStore, config: &DaemonConfig) -> Self {
        let paths = config.runtime_paths();
        Self {
            store,
            root: paths.git_mirror_root(),
            refresh_ttl: Duration::from_secs(config.git_mirror_refresh_ttl_seconds),
            max_unused: Duration::from_secs(config.git_mirror_max_unused_seconds),
            git_timeout: Duration::from_secs(config.git_operation_timeout_seconds),
            repo_locks: Arc::new(DashMap::new()),
        }
    }

    pub(crate) fn mirror_root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn mirror_key(repo_url: &str) -> String {
        let normalized = normalize_repo_url(repo_url);
        let digest = Sha256::digest(normalized.as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub(crate) fn mirror_dir_for_key(&self, mirror_key: &str) -> PathBuf {
        self.root.join(mirror_key)
    }

    pub(crate) async fn ensure_mirror(&self, repo_url: &str) -> anyhow::Result<PathBuf> {
        let repo_url = normalize_repo_url(repo_url);
        if repo_url.is_empty() {
            anyhow::bail!("repository URL must not be empty");
        }
        tokio::fs::create_dir_all(&self.root).await?;

        let mirror_key = Self::mirror_key(&repo_url);
        let mirror_dir = self.mirror_dir_for_key(&mirror_key);
        let lock = self.repo_lock(&mirror_key);
        let _guard = lock.lock().await;
        let now = unix_timestamp_now();

        let existing_state = self.store.get_git_mirror_cache_state(&mirror_key).await?;
        let mirror_exists = tokio::fs::try_exists(mirror_dir.join("HEAD")).await?;
        let mut last_fetched_at = existing_state
            .as_ref()
            .map(|state| state.last_fetched_at)
            .unwrap_or(0);

        if !mirror_exists {
            if tokio::fs::try_exists(&mirror_dir).await?
                && let Err(error) = tokio::fs::remove_dir_all(&mirror_dir).await
            {
                warn!(
                    ?error,
                    mirror_dir = %mirror_dir.display(),
                    "failed to remove stale git mirror directory before re-clone"
                );
            }
            self.clone_mirror(&repo_url, &mirror_dir).await?;
            last_fetched_at = now;
        } else {
            let should_refresh = existing_state
                .as_ref()
                .map(|state| {
                    now.saturating_sub(state.last_fetched_at) as u64 >= self.refresh_ttl.as_secs()
                })
                .unwrap_or(true);
            if should_refresh {
                self.refresh_mirror(&mirror_dir).await?;
                last_fetched_at = now;
            }
            if last_fetched_at == 0 {
                last_fetched_at = now;
            }
        }

        self.store
            .upsert_git_mirror_cache_state(&mirror_key, &repo_url, last_fetched_at, now)
            .await?;

        Ok(mirror_dir)
    }

    pub(crate) async fn cleanup_stale(&self) -> anyhow::Result<GitMirrorCleanupSummary> {
        tokio::fs::create_dir_all(&self.root).await?;
        let cutoff = unix_timestamp_now().saturating_sub(self.max_unused.as_secs() as i64);
        let stale_states = self
            .store
            .list_stale_git_mirror_cache_states(cutoff)
            .await?;

        let mut summary = GitMirrorCleanupSummary {
            scanned: stale_states.len(),
            removed: 0,
        };
        for state in stale_states {
            let path = self.mirror_dir_for_key(&state.mirror_key);
            if tokio::fs::try_exists(&path).await? {
                tokio::fs::remove_dir_all(&path).await.with_context(|| {
                    format!("failed to remove stale git mirror {}", path.display())
                })?;
            }
            self.store
                .delete_git_mirror_cache_state(&state.mirror_key)
                .await?;
            summary.removed += 1;
        }
        Ok(summary)
    }

    pub(crate) async fn stats(&self) -> anyhow::Result<GitMirrorCacheStatsSnapshot> {
        tokio::fs::create_dir_all(&self.root).await?;
        let cutoff = unix_timestamp_now().saturating_sub(self.max_unused.as_secs() as i64);
        let tracked_mirrors = self.store.count_git_mirror_cache_states().await?;
        let stale_mirrors = self
            .store
            .count_stale_git_mirror_cache_states(cutoff)
            .await?;
        let (latest_fetched_at, latest_used_at) =
            self.store.latest_git_mirror_cache_timestamps().await?;
        let mirror_directories = count_mirror_directories(&self.root).await?;
        Ok(GitMirrorCacheStatsSnapshot {
            mirror_root: self.root.clone(),
            refresh_ttl_seconds: self.refresh_ttl.as_secs(),
            max_unused_seconds: self.max_unused.as_secs(),
            tracked_mirrors,
            stale_mirrors,
            mirror_directories,
            latest_fetched_at,
            latest_used_at,
        })
    }

    fn repo_lock(&self, mirror_key: &str) -> Arc<Mutex<()>> {
        self.repo_locks
            .entry(mirror_key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    async fn clone_mirror(&self, repo_url: &str, mirror_dir: &Path) -> anyhow::Result<()> {
        if let Some(parent) = mirror_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        run_git(
            &[
                "clone",
                "--mirror",
                repo_url,
                mirror_dir.to_string_lossy().as_ref(),
            ],
            self.git_timeout,
        )
        .await
        .with_context(|| format!("failed to clone mirror for {}", repo_url))
    }

    async fn refresh_mirror(&self, mirror_dir: &Path) -> anyhow::Result<()> {
        run_git_with_git_dir(
            mirror_dir,
            &["remote", "update", "--prune"],
            self.git_timeout,
        )
        .await
        .with_context(|| format!("failed to refresh git mirror {}", mirror_dir.display()))
    }

    pub(crate) fn log_cleanup_result(&self, summary: GitMirrorCleanupSummary) {
        info!(
            mirror_root = %self.root.display(),
            scanned = summary.scanned,
            removed = summary.removed,
            "git mirror cache cleanup completed"
        );
    }
}

fn normalize_repo_url(repo_url: &str) -> String {
    repo_url.trim().trim_end_matches('/').to_string()
}

fn unix_timestamp_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

async fn run_git(args: &[&str], timeout: Duration) -> anyhow::Result<()> {
    run_git_command(Command::new("git").args(args), timeout).await
}

async fn run_git_with_git_dir(
    git_dir: &Path,
    args: &[&str],
    timeout: Duration,
) -> anyhow::Result<()> {
    let mut command = Command::new("git");
    command.arg("--git-dir").arg(git_dir).args(args);
    run_git_command(&mut command, timeout).await
}

async fn run_git_command(command: &mut Command, timeout: Duration) -> anyhow::Result<()> {
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .with_context(|| format!("git command timed out after {}s", timeout.as_secs()))??;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

async fn count_mirror_directories(root: &Path) -> anyhow::Result<usize> {
    let mut count = 0_usize;
    let mut entries = tokio::fs::read_dir(root).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            count += 1;
        }
    }
    Ok(count)
}
