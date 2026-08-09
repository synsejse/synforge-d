use std::path::{Path, PathBuf};

use glob::glob;
use synforge_core::package::{BuildEnvVar, PackageDefinition};
use tokio::process::Command;

use crate::logging::{BuildLogger, run_logged_command, run_mock_command};
use crate::meson::rewrite_meson_rust_wraps;

pub(super) async fn build_source_rpm(
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

pub(super) async fn run_mock_build(
    package: &PackageDefinition,
    mock_chroot: &str,
    srpm_path: &Path,
    topdir: &Path,
    mock_runtime_root: &Path,
    ccache_dir: &Path,
    ccache_stats_log: Option<&str>,
    logger: &BuildLogger,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(topdir).await?;
    let mock_lib_dir = mock_runtime_root.join("lib");
    let mock_cache_dir = mock_runtime_root.join("cache");
    tokio::fs::create_dir_all(&mock_lib_dir).await?;
    tokio::fs::create_dir_all(&mock_cache_dir).await?;
    logger.section("Run mock rebuild").await?;
    logger.line(format!("Target: {}", mock_chroot)).await?;
    let mut command = Command::new("mock");
    command
        .arg("-r")
        .arg(mock_chroot)
        .arg("--config-opts")
        .arg("use_bootstrap=False")
        .arg("--config-opts")
        .arg(format!("basedir={}", mock_lib_dir.display()))
        .arg("--config-opts")
        .arg(format!("cache_topdir={}", mock_cache_dir.display()))
        .arg("--isolation=simple")
        .arg("--resultdir")
        .arg(topdir);
    if package.ccache_enabled {
        tokio::fs::create_dir_all(ccache_dir).await?;
        logger
            .line(format!("Shared ccache: {}", ccache_dir.display()))
            .await?;
        command
            .arg("--enable-plugin")
            .arg("ccache")
            .arg("--plugin-option")
            .arg(format!("ccache:dir={}", ccache_dir.display()))
            .arg("--plugin-option")
            .arg("ccache:hashdir=False");
        if let Some(ccache_max_size_mb) = package.ccache_max_size_mb.filter(|value| *value > 0) {
            logger
                .line(format!("Shared ccache max size: {} MB", ccache_max_size_mb))
                .await?;
            command
                .arg("--plugin-option")
                .arg(format!("ccache:max_cache_size={}M", ccache_max_size_mb));
        }
    }
    if package.network_access {
        command.arg("--enable-network");
    }
    apply_mock_build_env(&mut command, &package.build_env, ccache_stats_log);
    command.arg("--rebuild").arg(srpm_path);
    run_mock_command(&mut command, logger, topdir).await
}

fn apply_build_env(command: &mut Command, build_env: &[BuildEnvVar]) {
    for entry in build_env {
        command.env(entry.key.trim(), entry.value.as_str());
    }
}

fn apply_mock_build_env(
    command: &mut Command,
    build_env: &[BuildEnvVar],
    ccache_stats_log: Option<&str>,
) {
    if build_env.is_empty() && ccache_stats_log.is_none() {
        return;
    }
    let mut exports = build_env
        .iter()
        .map(|entry| format!("export {}={};", entry.key.trim(), shell_quote(&entry.value)))
        .collect::<Vec<_>>();
    if let Some(path) = ccache_stats_log {
        exports.push(format!("export CCACHE_STATSLOG={};", shell_quote(path)));
    }
    let exports = exports.join(" ");

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
