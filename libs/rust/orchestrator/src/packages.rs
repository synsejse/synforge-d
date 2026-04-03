use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use synforge_core::{
    api::BrowseRepositoryResponse,
    config::DaemonConfig,
    model::{WorkerAction, WorkerJobPayload, WorkerParsePayload, WorkerResult},
    package::{BuildEnvVar, PackageDefinition, SpecRevision, SpecSource},
};
use tokio::process::Command;
use tracing::instrument;

use crate::workers::DockerWorkerLauncher;

#[derive(Debug, Clone)]
pub struct InspectedPackageSource {
    pub package_name: String,
    pub description: String,
    pub revision: SpecRevision,
}

#[derive(Clone)]
pub struct PackageSyncStore {
    root: PathBuf,
    config: DaemonConfig,
    worker_launcher: Arc<DockerWorkerLauncher>,
}

impl PackageSyncStore {
    pub fn new(root: PathBuf, config: DaemonConfig, worker_launcher: Arc<DockerWorkerLauncher>) -> Self {
        Self {
            root,
            config,
            worker_launcher,
        }
    }

    #[instrument(skip(self, source), fields(package_name = %package_name, repo_url = %source.repo_url))]
    pub async fn inspect_source(
        &self,
        package_name: &str,
        source: &SpecSource,
        timeout_seconds: u64,
    ) -> anyhow::Result<InspectedPackageSource> {
        let job_id = uuid::Uuid::now_v7();
        let paths = self.config.runtime_paths();
        let workspace_dir = paths.parse_workspace_dir(job_id);
        let artifact_dir = workspace_dir.join("artifacts");
        let payload = WorkerJobPayload {
            job_id,
            workspace_dir,
            artifact_dir,
            timeout_seconds,
            action: WorkerAction::Parse(WorkerParsePayload {
                package_name: package_name.to_string(),
                source: source.clone(),
            }),
        };
        let execution = self.worker_launcher.run_job(&payload, &self.config).await?;
        let WorkerResult::Parse(result) = execution.result else {
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
        enabled: bool,
        network_access: bool,
        mock_chroots: Vec<String>,
        poll_interval_seconds: u64,
        build_timeout_seconds: u64,
        package_history_count: u64,
        build_env: Vec<BuildEnvVar>,
    ) -> anyhow::Result<(PackageDefinition, SpecRevision)> {
        let inspected = self
            .inspect_source(package_name, source, build_timeout_seconds)
            .await?;
        let package = self
            .materialize(
                source,
                &inspected,
                enabled,
                network_access,
                mock_chroots,
                poll_interval_seconds,
                build_timeout_seconds,
                package_history_count,
                build_env,
            )
            .await?;
        Ok((package, inspected.revision.clone()))
    }

    pub async fn materialize(
        &self,
        source: &SpecSource,
        inspected: &InspectedPackageSource,
        enabled: bool,
        network_access: bool,
        mock_chroots: Vec<String>,
        poll_interval_seconds: u64,
        build_timeout_seconds: u64,
        package_history_count: u64,
        build_env: Vec<BuildEnvVar>,
    ) -> anyhow::Result<PackageDefinition> {
        let package = PackageDefinition {
            name: inspected.package_name.clone(),
            description: inspected.description.clone(),
            enabled,
            repo_subdir: inspected.package_name.clone(),
            publish_srpm: true,
            network_access,
            mock_chroots,
            source: source.clone(),
            poll_interval_seconds,
            build_timeout_seconds,
            package_history_count,
            build_env,
            spec_path: PathBuf::from(&source.spec_path),
            version: inspected.revision.version.clone(),
            release: inspected.revision.release.clone(),
        };
        package.validate()?;
        Ok(package)
    }

    pub async fn delete(&self, package_name: &str) -> anyhow::Result<()> {
        let path = self.root.join(package_name);
        if path.exists() {
            tokio::fs::remove_dir_all(&path)
                .await
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
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

        let paths = self.config.runtime_paths();
        tokio::fs::create_dir_all(paths.temp_root()).await?;
        let clone_dir = paths.browse_workspace_dir(uuid::Uuid::now_v7());

        let git_timeout = Duration::from_secs(self.config.git_operation_timeout_seconds);
        let clone_result = run_git(
            None,
            &[
            "clone",
            "--depth",
            "1",
            repo_url,
            clone_dir.to_string_lossy().as_ref(),
            ],
            git_timeout,
        )
        .await;

        let response = async {
            clone_result?;
            let head_commit = run_git(Some(&clone_dir), &["rev-parse", "HEAD"], git_timeout).await?;
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

            Ok(BrowseRepositoryResponse {
                repo_url: repo_url.to_string(),
                head_commit: head_commit.trim().to_string(),
                files,
                spec_files,
            })
        }
        .await;

        let _ = tokio::fs::remove_dir_all(&clone_dir).await;
        response
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
        .with_context(|| format!("git {} timed out after {}s", args.join(" "), timeout.as_secs()))??;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
