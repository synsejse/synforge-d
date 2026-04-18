use std::path::Path;

use tokio::process::Command;

use crate::logging::{BuildLogger, command_exists, run_logged_command};

pub(super) async fn prepare_build_tooling(logger: &BuildLogger) -> anyhow::Result<()> {
    if !command_exists("mock").await {
        logger
            .line("mock is not available in this worker image")
            .await?;
        return Ok(());
    }
    logger.line("mock is available").await?;
    Ok(())
}

pub(super) async fn fetch_spec_sources(
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

pub(super) async fn prepare_topdir(topdir: &Path) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(topdir.join("BUILD")).await?;
    tokio::fs::create_dir_all(topdir.join("BUILDROOT")).await?;
    tokio::fs::create_dir_all(topdir.join("RPMS")).await?;
    tokio::fs::create_dir_all(topdir.join("SOURCES")).await?;
    tokio::fs::create_dir_all(topdir.join("SPECS")).await?;
    tokio::fs::create_dir_all(topdir.join("SRPMS")).await?;
    Ok(())
}
