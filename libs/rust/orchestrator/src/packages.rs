use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use synforge_core::{
    api::{BrowseRepositoryProgressState, BrowseRepositoryProgressView, BrowseRepositoryResponse},
    config::DaemonConfig,
    constants::DEFAULT_WORKER_WORKSPACE_ROOT,
    error::SynforgeError,
    model::{WorkerAction, WorkerJobPayload, WorkerParsePayload, WorkerResult},
    package::{BuildEnvVar, PackageDefinition, SpecRevision, SpecSource},
    validation::{PackageDefinitionValidator, Validator},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::db::DieselStore;
use crate::git_cache::{GitMirrorCache, GitMirrorCacheStatsSnapshot};
use crate::workers::DockerWorkerLauncher;

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

#[derive(Clone)]
pub struct PackageSyncStore {
    config: DaemonConfig,
    worker_launcher: Arc<DockerWorkerLauncher>,
    git_mirror_cache: Arc<GitMirrorCache>,
    browse_progress: Arc<Mutex<Option<BrowseRepositoryProgressView>>>,
}

impl PackageSyncStore {
    pub fn new(
        config: DaemonConfig,
        worker_launcher: Arc<DockerWorkerLauncher>,
        store: DieselStore,
    ) -> Self {
        let git_mirror_cache = Arc::new(GitMirrorCache::new(store, &config));
        Self {
            config,
            worker_launcher,
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
        let execution = self.worker_launcher.run_job(&payload, &self.config).await?;
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

    #[instrument(skip(self), fields(repo_url = %repo_url))]
    pub async fn browse_repository(
        &self,
        repo_url: &str,
    ) -> anyhow::Result<BrowseRepositoryResponse> {
        let repo_url = repo_url.trim();
        if repo_url.is_empty() {
            anyhow::bail!("repository URL must not be empty");
        }
        let operation_id = uuid::Uuid::now_v7();
        {
            let mut slot = self.browse_progress.lock().await;
            if let Some(operation) = slot.as_ref()
                && operation.state == BrowseRepositoryProgressState::Running
            {
                return Err(anyhow::anyhow!(SynforgeError::Conflict(
                    "repository browse is already running".to_string()
                )));
            }
            *slot = Some(BrowseRepositoryProgressView {
                operation_id,
                repo_url: repo_url.to_string(),
                state: BrowseRepositoryProgressState::Running,
                progress_percent: 0,
                message: Some("Preparing repository clone…".to_string()),
            });
        }

        let paths = self.config.runtime_paths();
        let clone_dir = paths.repo_browse_workspace_dir(uuid::Uuid::now_v7());
        if let Some(parent) = clone_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let git_timeout = Duration::from_secs(self.config.git_operation_timeout_seconds);
        self.update_browse_progress(
            operation_id,
            repo_url,
            BrowseRepositoryProgressState::Running,
            5,
            Some("Checking mirror cache…".to_string()),
        )
        .await;
        let clone_result: anyhow::Result<()> =
            match self.git_mirror_cache.ensure_mirror(repo_url).await {
                Ok(mirror_dir) => {
                    let mirror_path = mirror_dir.to_string_lossy().to_string();
                    let mirror_clone = run_git_clone_with_progress(
                        None,
                        &[
                            "clone",
                            "--progress",
                            "--depth",
                            "1",
                            mirror_path.as_str(),
                            clone_dir.to_string_lossy().as_ref(),
                        ],
                        git_timeout,
                        |percent, message| {
                            self.update_browse_progress(
                                operation_id,
                                repo_url,
                                BrowseRepositoryProgressState::Running,
                                percent.min(95),
                                Some(message),
                            )
                        },
                    )
                    .await;
                    if let Err(error) = mirror_clone {
                        tracing::warn!(
                            repo_url,
                            mirror_dir = %mirror_dir.display(),
                            error = %error,
                            "failed to clone from mirror cache; falling back to direct clone"
                        );
                        if let Err(cleanup_error) = tokio::fs::remove_dir_all(&clone_dir).await {
                            tracing::warn!(
                                repo_url,
                                path = %clone_dir.display(),
                                error = %cleanup_error,
                                "failed to cleanup failed mirror clone workspace before retry"
                            );
                        }
                        self.update_browse_progress(
                            operation_id,
                            repo_url,
                            BrowseRepositoryProgressState::Running,
                            8,
                            Some("Mirror clone failed. Retrying direct clone…".to_string()),
                        )
                        .await;
                        run_git_clone_with_progress(
                            None,
                            &[
                                "clone",
                                "--progress",
                                "--depth",
                                "1",
                                repo_url,
                                clone_dir.to_string_lossy().as_ref(),
                            ],
                            git_timeout,
                            |percent, message| {
                                self.update_browse_progress(
                                    operation_id,
                                    repo_url,
                                    BrowseRepositoryProgressState::Running,
                                    percent.min(95),
                                    Some(message),
                                )
                            },
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        repo_url,
                        error = %error,
                        "failed to prepare mirror cache; falling back to direct clone"
                    );
                    self.update_browse_progress(
                        operation_id,
                        repo_url,
                        BrowseRepositoryProgressState::Running,
                        8,
                        Some("Mirror cache unavailable. Cloning directly…".to_string()),
                    )
                    .await;
                    run_git_clone_with_progress(
                        None,
                        &[
                            "clone",
                            "--progress",
                            "--depth",
                            "1",
                            repo_url,
                            clone_dir.to_string_lossy().as_ref(),
                        ],
                        git_timeout,
                        |percent, message| {
                            self.update_browse_progress(
                                operation_id,
                                repo_url,
                                BrowseRepositoryProgressState::Running,
                                percent.min(95),
                                Some(message),
                            )
                        },
                    )
                    .await
                }
            };

        let response: anyhow::Result<BrowseRepositoryResponse> = async {
            clone_result?;
            self.update_browse_progress(
                operation_id,
                repo_url,
                BrowseRepositoryProgressState::Running,
                96,
                Some("Reading repository metadata…".to_string()),
            )
            .await;
            let head_commit =
                run_git(Some(&clone_dir), &["rev-parse", "HEAD"], git_timeout).await?;
            self.update_browse_progress(
                operation_id,
                repo_url,
                BrowseRepositoryProgressState::Running,
                98,
                Some("Scanning repository files…".to_string()),
            )
            .await;
            let files_output = run_git(Some(&clone_dir), &["ls-files"], git_timeout).await?;
            let mut files = files_output
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            files.sort();
            let spec_files = files
                .iter()
                .filter(|path| path.ends_with(".spec"))
                .cloned()
                .collect::<Vec<_>>();

            let response = BrowseRepositoryResponse {
                repo_url: repo_url.to_string(),
                head_commit: head_commit.trim().to_string(),
                files,
                spec_files,
            };
            self.update_browse_progress(
                operation_id,
                repo_url,
                BrowseRepositoryProgressState::Completed,
                100,
                Some(format!(
                    "Clone complete. {} spec file(s) discovered.",
                    response.spec_files.len()
                )),
            )
            .await;
            Ok(response)
        }
        .await;

        if let Err(cleanup_error) = tokio::fs::remove_dir_all(&clone_dir).await {
            tracing::warn!(
                repo_url,
                path = %clone_dir.display(),
                error = %cleanup_error,
                "failed to cleanup repository browse workspace"
            );
        }
        if let Err(error) = &response {
            self.update_browse_progress(
                operation_id,
                repo_url,
                BrowseRepositoryProgressState::Failed,
                100,
                Some(error.to_string()),
            )
            .await;
        }
        response
    }

    async fn update_browse_progress(
        &self,
        operation_id: uuid::Uuid,
        repo_url: &str,
        state: BrowseRepositoryProgressState,
        progress_percent: u8,
        message: Option<String>,
    ) {
        let mut slot = self.browse_progress.lock().await;
        *slot = Some(BrowseRepositoryProgressView {
            operation_id,
            repo_url: repo_url.to_string(),
            state,
            progress_percent,
            message,
        });
    }
}

async fn run_git(dir: Option<&Path>, args: &[&str], timeout: Duration) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn run_git_clone_with_progress<F, Fut>(
    dir: Option<&Path>,
    args: &[&str],
    timeout: Duration,
    mut on_progress: F,
) -> anyhow::Result<()>
where
    F: FnMut(u8, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn git clone command: git {}", args.join(" ")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture git clone stderr"))?;
    let mut stderr_lines = BufReader::new(stderr).lines();
    let mut stderr_output = String::new();

    loop {
        let next_line = tokio::time::timeout(timeout, stderr_lines.next_line())
            .await
            .with_context(|| {
                format!(
                    "git {} timed out after {}s",
                    args.join(" "),
                    timeout.as_secs()
                )
            })??;
        let Some(line) = next_line else {
            break;
        };
        if !stderr_output.is_empty() {
            stderr_output.push('\n');
        }
        stderr_output.push_str(&line);
        if let Some((progress_percent, message)) = parse_git_clone_progress_line(&line) {
            on_progress(progress_percent, message).await;
        }
    }

    let status = tokio::time::timeout(timeout, child.wait())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !status.success() {
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr_output);
    }
    Ok(())
}

fn parse_git_clone_progress_line(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim();
    if trimmed.starts_with("Cloning into") {
        return Some((2, "Starting clone…".to_string()));
    }
    if trimmed.contains("Checking connectivity") {
        return Some((99, "Checking repository connectivity…".to_string()));
    }

    for (marker, stage) in [
        ("Receiving objects:", "Receiving objects"),
        ("Resolving deltas:", "Resolving deltas"),
        ("Compressing objects:", "Compressing objects"),
    ] {
        let Some((_, rest)) = trimmed.split_once(marker) else {
            continue;
        };
        let Some(progress_percent) = extract_progress_percent(rest) else {
            continue;
        };
        return Some((progress_percent, format!("{stage}… {progress_percent}%")));
    }
    None
}

fn extract_progress_percent(segment: &str) -> Option<u8> {
    let percent_index = segment.find('%')?;
    let token = segment[..percent_index].split_whitespace().last()?;
    let digits = token
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u8>().ok().map(|value| value.min(100))
}
