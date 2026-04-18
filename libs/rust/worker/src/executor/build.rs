use std::path::Path;

use synforge_core::{
    model::{BuildArtifact, BuildStatus, WorkerBuildPayload, WorkerBuildResult, WorkerJobPayload},
    package::PackageDefinition,
};
use tracing::{info, warn};

use super::{
    build_source_rpm, fetch_spec_sources, prepare_build_tooling, prepare_topdir, run_mock_build,
};
use crate::artifact::{collect_artifacts, collect_success_artifacts};
use crate::git::clone_repository;
use crate::logging::{BuildLogger, log_best_effort};
use crate::meson::apply_spec_compat_fixes;
use crate::protocol::WorkerTransportHandle;

pub(super) async fn execute_spec_build(
    package: &PackageDefinition,
    payload: &WorkerJobPayload,
    build_payload: &WorkerBuildPayload,
    transport: Option<WorkerTransportHandle>,
) -> anyhow::Result<WorkerBuildResult> {
    info!(
        job_id = %payload.job_id,
        package_name = %package.name,
        mock_chroot = %build_payload.mock_chroot,
        revision = %build_payload.revision,
        checkout_commit = ?build_payload.checkout_commit,
        timeout_seconds = payload.timeout_seconds,
        "starting RPM build execution"
    );
    let logs_dir = payload.workspace_dir.join("logs");
    let logger = BuildLogger::new(&logs_dir, transport).await?;
    let topdir = payload.workspace_dir.join("rpmbuild");
    let repo_dir = payload.workspace_dir.join("repo");
    clone_repository(
        &package.source,
        &repo_dir,
        build_payload.checkout_commit.as_deref(),
    )
    .await?;
    info!(
        job_id = %payload.job_id,
        package_name = %package.name,
        repo_url = %package.source.repo_url,
        "repository checkout prepared"
    );
    let spec_file = repo_dir.join(&package.source.spec_file);
    let package_dir = spec_file.parent().map(Path::to_path_buf).ok_or_else(|| {
        anyhow::anyhow!("spec path {} has no parent directory", spec_file.display())
    })?;
    let build = async {
        logger.section("Prepare build workspace").await?;
        logger.line(format!("Package: {}", package.name)).await?;
        logger
            .line(format!("Spec file: {}", spec_file.display()))
            .await?;
        logger
            .line(format!("Repository: {}", package.source.repo_url))
            .await?;
        if let Some(checkout_commit) = &build_payload.checkout_commit {
            logger.line(format!("Commit: {}", checkout_commit)).await?;
        }
        logger
            .line(format!("Workspace: {}", payload.workspace_dir.display()))
            .await?;
        logger.section("Apply spec compatibility fixes").await?;
        apply_spec_compat_fixes(&spec_file).await?;
        logger.section("Prepare chroot build tooling").await?;
        prepare_build_tooling(&logger).await?;
        logger.section("Fetch sources").await?;
        fetch_spec_sources(&spec_file, &package_dir, &logger).await?;
        let source_topdir = topdir.join("source");
        prepare_topdir(&source_topdir).await?;
        let srpm_path =
            build_source_rpm(package, &spec_file, &package_dir, &source_topdir, &logger).await?;
        let mut artifacts = Vec::new();
        let arch_topdir = topdir.join(&build_payload.mock_chroot);
        let mock_runtime_root = payload.workspace_dir.join("mock");
        let ccache_dir = payload.workspace_dir.join("ccache");
        logger.section("Build packages").await?;
        logger
            .line(format!("Target: {}", build_payload.mock_chroot))
            .await?;
        run_mock_build(
            package,
            &build_payload.mock_chroot,
            &srpm_path,
            &arch_topdir,
            &mock_runtime_root,
            &ccache_dir,
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
            info!(
                job_id = %payload.job_id,
                package_name = %package.name,
                mock_chroot = %build_payload.mock_chroot,
                artifact_count = artifacts.len(),
                "RPM build completed successfully"
            );
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::Succeeded,
                artifacts,
                message: None,
            })
        }
        Ok(Err(error)) => {
            let message = error.to_string();
            warn!(
                job_id = %payload.job_id,
                package_name = %package.name,
                mock_chroot = %build_payload.mock_chroot,
                error = %message,
                "RPM build failed"
            );
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
                message: Some(combine_messages(message, artifact_message)),
            })
        }
        Err(_) => {
            warn!(
                job_id = %payload.job_id,
                package_name = %package.name,
                mock_chroot = %build_payload.mock_chroot,
                timeout_seconds = payload.timeout_seconds,
                "RPM build timed out"
            );
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
                message: Some(combine_messages(
                    "build timed out".to_string(),
                    artifact_message,
                )),
            })
        }
    }
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
