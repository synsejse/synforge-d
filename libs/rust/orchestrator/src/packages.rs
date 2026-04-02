use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use synforge_core::{
    BuildEnvVar, DaemonConfig, PackageDefinition, SpecRevision, SpecSource, WorkerAction,
    WorkerJobPayload, WorkerResult,
};
use tokio::process::Command;

use crate::workers::WorkerLauncher;

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
    worker_launcher: Arc<dyn WorkerLauncher>,
}

impl PackageSyncStore {
    pub fn new(root: PathBuf, config: DaemonConfig, worker_launcher: Arc<dyn WorkerLauncher>) -> Self {
        Self {
            root,
            config,
            worker_launcher,
        }
    }

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
            action: WorkerAction::Parse(synforge_core::WorkerParsePayload {
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

    pub async fn browse_repository(
        &self,
        repo_url: &str,
    ) -> anyhow::Result<synforge_core::BrowseRepositoryResponse> {
        let repo_url = repo_url.trim();
        if repo_url.is_empty() {
            anyhow::bail!("repository URL must not be empty");
        }

        let paths = self.config.runtime_paths();
        tokio::fs::create_dir_all(paths.temp_root()).await?;
        let clone_dir = paths.browse_workspace_dir(uuid::Uuid::now_v7());

        let clone_result = run_git(&[
            "clone",
            "--depth",
            "1",
            repo_url,
            clone_dir.to_string_lossy().as_ref(),
        ])
        .await;

        let response = async {
            clone_result?;
            let head_commit = run_git_in(&clone_dir, &["rev-parse", "HEAD"]).await?;
            let files_output = run_git_in(&clone_dir, &["ls-files"]).await?;
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

            Ok(synforge_core::BrowseRepositoryResponse {
                repo_url: repo_url.to_string(),
                head_commit: head_commit.trim().to_string(),
                files,
                spec_files,
            })
        }
        .await;

        tokio::fs::remove_dir_all(&clone_dir).await.ok();
        response
    }
}

async fn run_git(args: &[&str]) -> anyhow::Result<String> {
    run_git_command(None, args).await
}

async fn run_git_in(dir: &std::path::Path, args: &[&str]) -> anyhow::Result<String> {
    run_git_command(Some(dir), args).await
}

async fn run_git_command(dir: Option<&std::path::Path>, args: &[&str]) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = command.output().await?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
