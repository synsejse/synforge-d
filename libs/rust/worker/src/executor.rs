use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use glob::glob;
use sha2::{Digest, Sha256};
use synforge_core::{
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, WorkerBuildPayload, WorkerBuildResult,
        WorkerJobPayload,
    },
    package::{BuildEnvVar, PackageDefinition},
};
use tokio::process::Command;

use crate::git::clone_repository;
use crate::logging::{
    BuildLogger, command_exists, log_best_effort, run_logged_command, run_mock_command,
};
use crate::meson::{apply_spec_compat_fixes, rewrite_meson_rust_wraps};
use crate::protocol::WorkerTransportHandle;

#[async_trait]
pub trait BuildExecutor: Send + Sync {
    async fn execute_build(
        &self,
        package: &PackageDefinition,
        payload: &WorkerJobPayload,
        build: &WorkerBuildPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerBuildResult>;
}

#[derive(Debug, Default, Clone)]
pub struct RpmBuildExecutor;

#[async_trait]
impl BuildExecutor for RpmBuildExecutor {
    async fn execute_build(
        &self,
        package: &PackageDefinition,
        payload: &WorkerJobPayload,
        build: &WorkerBuildPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerBuildResult> {
        execute_spec_build(package, payload, build, transport).await
    }
}

async fn execute_spec_build(
    package: &PackageDefinition,
    payload: &WorkerJobPayload,
    build_payload: &WorkerBuildPayload,
    transport: Option<WorkerTransportHandle>,
) -> anyhow::Result<WorkerBuildResult> {
    let logs_dir = payload.workspace_dir.join("logs");
    let logger = BuildLogger::new(&logs_dir, transport).await?;
    let topdir = payload.workspace_dir.join("rpmbuild");
    let repo_dir = payload.workspace_dir.join("repo");
    clone_repository(
        &package.source,
        &repo_dir,
        build_payload.source_commit.as_deref(),
    )
    .await?;
    let spec_path = repo_dir.join(&package.source.spec_path);
    let package_dir = spec_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        anyhow::anyhow!("spec path {} has no parent directory", spec_path.display())
    })?;
    let build = async {
        logger.section("Prepare build workspace").await?;
        logger.line(format!("Package: {}", package.name)).await?;
        logger
            .line(format!("Spec path: {}", spec_path.display()))
            .await?;
        logger
            .line(format!("Repository: {}", package.source.repo_url))
            .await?;
        if let Some(source_commit) = &build_payload.source_commit {
            logger.line(format!("Commit: {}", source_commit)).await?;
        }
        logger
            .line(format!("Workspace: {}", payload.workspace_dir.display()))
            .await?;
        logger.section("Apply spec compatibility fixes").await?;
        apply_spec_compat_fixes(&spec_path).await?;
        logger.section("Prepare chroot build tooling").await?;
        prepare_build_tooling(&logger).await?;
        logger.section("Fetch sources").await?;
        fetch_spec_sources(&spec_path, &package_dir, &logger).await?;
        let source_topdir = topdir.join("source");
        prepare_topdir(&source_topdir).await?;
        let srpm_path =
            build_source_rpm(package, &spec_path, &package_dir, &source_topdir, &logger).await?;
        let mut artifacts = Vec::new();
        let arch_topdir = topdir.join(&build_payload.mock_chroot);
        logger.section("Build packages").await?;
        logger
            .line(format!("Target: {}", build_payload.mock_chroot))
            .await?;
        run_mock_build(
            package,
            &build_payload.mock_chroot,
            &srpm_path,
            &arch_topdir,
            &logger,
        )
        .await?;
        artifacts.extend(
            collect_success_artifacts(
                package,
                &arch_topdir,
                &source_topdir,
                &build_payload.mock_chroot,
            )
            .await?,
        );
        logger.section("Collect artifacts").await?;
        logger
            .line(format!("Artifacts collected: {}", artifacts.len()))
            .await?;
        anyhow::Ok(artifacts)
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(payload.timeout_seconds),
        build,
    )
    .await
    {
        Ok(Ok(artifacts)) => {
            logger.section("Build completed").await?;
            logger.line("Build finished successfully").await?;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::Succeeded,
                artifacts,
                logs_path: None,
                message: None,
            })
        }
        Ok(Err(error)) => {
            let message = error.to_string();
            log_best_effort(&logger, "Build failed", &message).await;
            let (artifacts, artifact_message) = collect_artifacts_after_failure(
                package,
                &topdir,
                &build_payload.mock_chroot,
                &logger,
            )
            .await;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::Failed,
                artifacts,
                logs_path: None,
                message: Some(combine_messages(message, artifact_message)),
            })
        }
        Err(_) => {
            log_best_effort(&logger, "Build timed out", "build timed out").await;
            let (artifacts, artifact_message) = collect_artifacts_after_failure(
                package,
                &topdir,
                &build_payload.mock_chroot,
                &logger,
            )
            .await;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::TimedOut,
                artifacts,
                logs_path: None,
                message: Some(combine_messages(
                    "build timed out".to_string(),
                    artifact_message,
                )),
            })
        }
    }
}

async fn prepare_build_tooling(logger: &BuildLogger) -> anyhow::Result<()> {
    if !command_exists("mock").await {
        logger
            .line("mock is not available in this worker image")
            .await?;
        return Ok(());
    }
    logger.line("mock is available").await?;
    Ok(())
}

async fn fetch_spec_sources(
    spec_path: &Path,
    package_dir: &Path,
    logger: &BuildLogger,
) -> anyhow::Result<()> {
    if !command_exists("spectool").await {
        logger
            .line("Skipping source fetch: spectool not available")
            .await?;
        return Ok(());
    }

    let spec_name = spec_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("spec path {} has no file name", spec_path.display()))?;

    run_logged_command(
        Command::new("spectool")
            .current_dir(package_dir)
            .arg("-g")
            .arg("-R")
            .arg("--define")
            .arg(format!("_sourcedir {}", package_dir.display()))
            .arg(spec_name),
        logger,
    )
    .await
}

async fn build_source_rpm(
    package: &PackageDefinition,
    spec_path: &Path,
    package_dir: &Path,
    topdir: &Path,
    logger: &BuildLogger,
) -> anyhow::Result<PathBuf> {
    logger.section("Rewrite Meson Rust wraps").await?;
    rewrite_meson_rust_wraps(package_dir, logger).await?;
    logger.section("Create source RPM").await?;
    let mut command = Command::new("rpmbuild");
    command
        .arg("-vv")
        .arg("--define")
        .arg(format!("_topdir {}", topdir.display()))
        .arg("--define")
        .arg(format!("_sourcedir {}", package_dir.display()))
        .arg("--define")
        .arg(format!("_specdir {}", package_dir.display()))
        .arg("-bs")
        .arg(spec_path);
    apply_build_env(&mut command, &package.build_env);
    run_logged_command(&mut command, logger).await?;

    let mut matches = glob(&format!("{}/SRPMS/*.src.rpm", topdir.display()))?
        .filter_map(Result::ok)
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    matches.sort();
    matches
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("source RPM was not produced"))
}

async fn run_mock_build(
    package: &PackageDefinition,
    mock_chroot: &str,
    srpm_path: &Path,
    topdir: &Path,
    logger: &BuildLogger,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(topdir).await?;
    logger.section("Run mock rebuild").await?;
    logger.line(format!("Target: {}", mock_chroot)).await?;
    let mut command = Command::new("mock");
    command
        .arg("-r")
        .arg(mock_chroot)
        .arg("--config-opts")
        .arg("use_bootstrap=False")
        .arg("--isolation=simple")
        .arg("--resultdir")
        .arg(topdir);
    if package.network_access {
        command.arg("--enable-network");
    }
    apply_mock_build_env(&mut command, &package.build_env);
    command.arg("--rebuild").arg(srpm_path);
    run_mock_command(&mut command, logger, topdir).await
}

fn apply_build_env(command: &mut Command, build_env: &[BuildEnvVar]) {
    for entry in build_env {
        command.env(entry.key.trim(), entry.value.as_str());
    }
}

fn apply_mock_build_env(command: &mut Command, build_env: &[BuildEnvVar]) {
    if build_env.is_empty() {
        return;
    }
    let exports = build_env
        .iter()
        .map(|entry| format!("export {}={};", entry.key.trim(), shell_quote(&entry.value)))
        .collect::<Vec<_>>()
        .join(" ");

    for macro_name in [
        "__spec_prep_pre",
        "__spec_build_pre",
        "__spec_install_pre",
        "__spec_check_pre",
        "__spec_clean_pre",
    ] {
        command
            .arg("--define")
            .arg(format!(r#"{macro_name} {exports} %{{___build_pre}}"#));
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

async fn collect_artifacts(
    package: &PackageDefinition,
    topdir: &Path,
    mock_chroot: &str,
) -> anyhow::Result<Vec<BuildArtifact>> {
    let patterns = [format!("{}/**/*.rpm", topdir.display())];
    let mut artifacts = Vec::new();
    let mut seen = BTreeSet::new();
    for pattern in patterns {
        for entry in glob(&pattern)? {
            let path = entry?;
            if path.is_file() {
                let artifact = build_artifact(package, topdir, path, mock_chroot).await?;
                if seen.insert(artifact.relative_repo_path.clone()) {
                    artifacts.push(artifact);
                }
            }
        }
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

async fn collect_success_artifacts(
    package: &PackageDefinition,
    mock_topdir: &Path,
    source_topdir: &Path,
    mock_chroot: &str,
) -> anyhow::Result<Vec<BuildArtifact>> {
    let mut artifacts = collect_artifacts(package, mock_topdir, mock_chroot).await?;
    let has_srpm = artifacts
        .iter()
        .any(|artifact| artifact.kind == ArtifactKind::Srpm);
    if !has_srpm {
        let mut source_artifacts = collect_artifacts(package, source_topdir, mock_chroot)
            .await
            .with_context(|| {
                format!(
                    "failed to collect source artifacts from {}",
                    source_topdir.display()
                )
            })?;
        source_artifacts.retain(|artifact| artifact.kind == ArtifactKind::Srpm);
        artifacts.extend(source_artifacts);
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    }
    Ok(artifacts)
}

async fn prepare_topdir(topdir: &Path) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(topdir.join("BUILD")).await?;
    tokio::fs::create_dir_all(topdir.join("BUILDROOT")).await?;
    tokio::fs::create_dir_all(topdir.join("RPMS")).await?;
    tokio::fs::create_dir_all(topdir.join("SOURCES")).await?;
    tokio::fs::create_dir_all(topdir.join("SPECS")).await?;
    tokio::fs::create_dir_all(topdir.join("SRPMS")).await?;
    Ok(())
}

fn artifact_rpm_arch(path: &Path) -> Option<&str> {
    let filename = path.file_name()?.to_str()?;
    if filename.ends_with(".src.rpm") {
        return Some("src");
    }
    let name = filename.strip_suffix(".rpm")?;
    name.rsplit('.').next()
}

async fn build_artifact(
    package: &PackageDefinition,
    topdir: &Path,
    path: PathBuf,
    mock_chroot: &str,
) -> anyhow::Result<BuildArtifact> {
    let bytes = tokio::fs::read(&path)
        .await
        .with_context(|| format!("failed to read artifact {}", path.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let kind = if path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.ends_with(".src.rpm"))
    {
        ArtifactKind::Srpm
    } else {
        ArtifactKind::Rpm
    };
    Ok(BuildArtifact {
        package_name: package.name.clone(),
        mock_chroot: mock_chroot.to_string(),
        arch: artifact_rpm_arch(&path).unwrap_or("unknown").to_string(),
        size_bytes: bytes.len() as u64,
        relative_repo_path: path
            .strip_prefix(topdir)
            .ok()
            .map(PathBuf::from)
            .or_else(|| path.file_name().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("artifact.rpm")),
        path,
        sha256,
        kind,
    })
}

async fn collect_artifacts_after_failure(
    package: &PackageDefinition,
    topdir: &Path,
    mock_chroot: &str,
    logger: &BuildLogger,
) -> (Vec<BuildArtifact>, Option<String>) {
    match collect_artifacts(package, topdir, mock_chroot).await {
        Ok(artifacts) => (artifacts, None),
        Err(error) => {
            let message = format!("artifact collection failed: {}", error);
            log_best_effort(logger, "Artifact collection failed", &message).await;
            (Vec::new(), Some(message))
        }
    }
}

fn combine_messages(message: String, extra: Option<String>) -> String {
    match extra {
        Some(extra) => format!("{message}; {extra}"),
        None => message,
    }
}
