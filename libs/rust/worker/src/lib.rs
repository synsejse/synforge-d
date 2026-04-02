use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use glob::glob;
use ini::Ini;
use sha2::{Digest, Sha256};
use synforge_core::{
    ArtifactKind, BuildArtifact, BuildStatus, PackageDefinition, ParsedSpec, SpecRevision,
    SpecSource, SynforgeError, WorkerAction, WorkerBuildPayload, WorkerBuildResult,
    WorkerJobPayload, WorkerParsePayload, WorkerParseResult, WorkerResult, WorkerWireMessage,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::warn;

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

#[derive(Debug, Default, Clone)]
pub struct WorkerRuntime<E = RpmBuildExecutor> {
    executor: E,
}

impl WorkerRuntime<RpmBuildExecutor> {
    pub fn new() -> Self {
        Self {
            executor: RpmBuildExecutor,
        }
    }
}

impl<E> WorkerRuntime<E>
where
    E: BuildExecutor,
{
    pub async fn run_from_env(&self) -> anyhow::Result<WorkerResult> {
        let worker_id = env_required("SYNFORGE_WORKER_ID")?;
        let connect_addr = env_required("SYNFORGE_WORKER_CONNECT_ADDR")?;
        let transport = WorkerTransportHandle::connect(&connect_addr, &worker_id).await?;
        let payload = transport.receive_assignment().await?;
        self.run_with_transport(payload, Some(transport)).await
    }

    pub async fn run(&self, payload: WorkerJobPayload) -> anyhow::Result<WorkerResult> {
        self.run_with_transport(payload, None).await
    }

    async fn run_with_transport(
        &self,
        payload: WorkerJobPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerResult> {
        tokio::fs::create_dir_all(&payload.workspace_dir).await?;
        tokio::fs::create_dir_all(&payload.artifact_dir).await?;
        let local_result = match &payload.action {
            WorkerAction::Parse(parse) => WorkerResult::Parse(execute_spec_parse(&payload, parse).await?),
            WorkerAction::Build(build) => {
                build
                    .package
                    .validate()
                    .map_err(|error: SynforgeError| anyhow::anyhow!(error.to_string()))?;
                WorkerResult::Build(
                    self.executor
                        .execute_build(&build.package, &payload, build, transport.clone())
                        .await?,
                )
            }
        };
        publish_worker_result(transport.as_ref(), &local_result).await?;
        Ok(local_result)
    }
}

fn env_required(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("missing required env var {}", name))
}

async fn publish_worker_result(
    transport: Option<&WorkerTransportHandle>,
    result: &WorkerResult,
) -> anyhow::Result<()> {
    let Some(transport) = transport else {
        return Ok(());
    };

    let uploaded_result = match result {
        WorkerResult::Parse(parse) => WorkerResult::Parse(parse.clone()),
        WorkerResult::Build(build) => {
            for artifact in &build.artifacts {
                transport.send_artifact(artifact).await?;
            }

            WorkerResult::Build(WorkerBuildResult {
                artifacts: Vec::new(),
                logs_path: None,
                ..build.clone()
            })
        }
    };
    transport.send_result(uploaded_result).await?;
    Ok(())
}

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

async fn execute_spec_parse(
    job_payload: &WorkerJobPayload,
    payload: &WorkerParsePayload,
) -> anyhow::Result<WorkerParseResult> {
    let repo_dir = job_payload.workspace_dir.join("repo");
    clone_repository(&payload.source, &repo_dir, None).await?;
    let spec_path = repo_dir.join(&payload.source.spec_path);
    if !tokio::fs::try_exists(&spec_path).await? {
        anyhow::bail!(
            "spec file {} does not exist in repository {}",
            payload.source.spec_path,
            payload.source.repo_url
        );
    }
    let parsed = query_spec_metadata(&spec_path).await?;
    let commit = git_rev_parse(&repo_dir, "HEAD").await?;
    Ok(WorkerParseResult {
        parsed: parsed.clone(),
        revision: SpecRevision::new(
            parsed.version,
            parsed.release,
            Some(commit),
        ),
    })
}

async fn execute_spec_build(
    package: &PackageDefinition,
    payload: &WorkerJobPayload,
    build_payload: &WorkerBuildPayload,
    transport: Option<WorkerTransportHandle>,
) -> anyhow::Result<WorkerBuildResult> {
    let log_path = payload.workspace_dir.join("build.log");
    let logger = LiveLogSink::new(&log_path, transport).await?;
    let topdir = payload.workspace_dir.join("rpmbuild");
    let repo_dir = payload.workspace_dir.join("repo");
    clone_repository(
        &package.source,
        &repo_dir,
        build_payload.source_commit.as_deref(),
    )
    .await?;
    let spec_path = repo_dir.join(&package.source.spec_path);
    let package_dir = spec_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("spec path {} has no parent directory", spec_path.display()))?;
    let build = async {
        logger.section("Prepare build workspace").await?;
        logger
            .line(format!("Package: {}", package.name))
            .await?;
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

    match tokio::time::timeout(std::time::Duration::from_secs(payload.timeout_seconds), build).await {
        Ok(Ok(artifacts)) => {
            logger.section("Build completed").await?;
            logger.line("Build finished successfully").await?;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::Succeeded,
                artifacts,
                logs_path: Some(log_path),
                message: None,
            })
        }
        Ok(Err(error)) => {
            let message = error.to_string();
            log_best_effort(&logger, "Build failed", &message).await;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::Failed,
                artifacts: collect_artifacts(package, &topdir, &build_payload.mock_chroot)
                    .await
                    .unwrap_or_default(),
                logs_path: Some(log_path),
                message: Some(message),
            })
        }
        Err(_) => {
            log_best_effort(&logger, "Build timed out", "build timed out").await;
            Ok(WorkerBuildResult {
                job_id: payload.job_id,
                package_name: package.name.clone(),
                status: BuildStatus::TimedOut,
                artifacts: collect_artifacts(package, &topdir, &build_payload.mock_chroot)
                    .await
                    .unwrap_or_default(),
                logs_path: Some(log_path),
                message: Some("build timed out".to_string()),
            })
        }
    }
}

async fn log_best_effort(logger: &LiveLogSink, section: &str, message: &str) {
    if let Err(error) = logger.section(section).await {
        warn!("failed to write worker log section: {}", error);
    }
    if let Err(error) = logger.line(message).await {
        warn!("failed to write worker log line: {}", error);
    }
}

async fn apply_spec_compat_fixes(spec_path: &Path) -> anyhow::Result<()> {
    let mut contents = tokio::fs::read_to_string(spec_path).await?;

    if contents.contains("crate(rustc-hash)")
        && contents.contains("%define rewrite_wrap_file()")
        && !contents.contains("%rewrite_wrap_file rustc-hash")
    {
        let needle = "%rewrite_wrap_file unicode-ident\n";
        if contents.contains(needle) {
            contents = contents.replace(
                needle,
                "%rewrite_wrap_file unicode-ident\n%rewrite_wrap_file rustc-hash\n",
            );
            tokio::fs::write(spec_path, contents).await?;
        }
    }

    Ok(())
}

async fn query_spec_metadata(spec_path: &Path) -> anyhow::Result<ParsedSpec> {
    let output = Command::new("rpmspec")
        .arg("-q")
        .arg("--srpm")
        .arg("--qf")
        .arg("%{name}\n%{version}\n%{release}\n%{summary}\n")
        .arg(spec_path)
        .output()
        .await
        .with_context(|| format!("failed to run rpmspec for {}", spec_path.display()))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "rpmspec failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout).context("rpmspec output was not valid UTF-8")?;
    let mut lines = stdout.lines();
    let name = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("rpmspec did not output a name"))?
        .trim()
        .to_string();
    let version = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("rpmspec did not output a version"))?
        .trim()
        .to_string();
    let release = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("rpmspec did not output a release"))?
        .trim()
        .to_string();
    let summary = lines.next().unwrap_or_default().trim().to_string();

    Ok(ParsedSpec {
        name,
        version,
        release,
        summary,
    })
}

async fn clone_repository(
    source: &SpecSource,
    destination: &Path,
    commit: Option<&str>,
) -> anyhow::Result<()> {
    if tokio::fs::try_exists(destination).await? {
        tokio::fs::remove_dir_all(destination).await.ok();
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    run_command(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(&source.repo_url)
            .arg(destination),
    )
    .await
    .with_context(|| format!("failed to clone {}", source.repo_url))?;

    if let Some(commit) = commit {
        run_command(
            Command::new("git")
                .current_dir(destination)
                .arg("fetch")
                .arg("--depth")
                .arg("1")
                .arg("origin")
                .arg(commit),
        )
        .await
        .with_context(|| format!("failed to fetch commit {}", commit))?;
        run_command(
            Command::new("git")
                .current_dir(destination)
                .arg("checkout")
                .arg("--detach")
                .arg(commit),
        )
        .await
        .with_context(|| format!("failed to checkout commit {}", commit))?;
    }
    Ok(())
}

async fn git_rev_parse(repo_dir: &Path, rev: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .arg("rev-parse")
        .arg(rev)
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse {} failed: {}",
            rev,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn run_command(command: &mut Command) -> anyhow::Result<()> {
    let output = command.output().await?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}

async fn prepare_build_tooling(logger: &LiveLogSink) -> anyhow::Result<()> {
    if !command_exists("mock").await {
        logger.line("mock is not available in this worker image").await?;
        return Ok(());
    }
    logger.line("mock is available").await?;
    Ok(())
}

async fn fetch_spec_sources(spec_path: &Path, package_dir: &Path, logger: &LiveLogSink) -> anyhow::Result<()> {
    if !command_exists("spectool").await {
        logger.line("Skipping source fetch: spectool not available").await?;
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
    logger: &LiveLogSink,
) -> anyhow::Result<std::path::PathBuf> {
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
    logger: &LiveLogSink,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(topdir).await?;
    logger.section("Run mock rebuild").await?;
    logger.line(format!("Target: {}", mock_chroot)).await?;
    let mut command = Command::new("mock");
    command
        .arg("-v")
        .arg("-r")
        .arg(mock_chroot)
        .arg("--config-opts")
        .arg("use_bootstrap=False")
        .arg("--isolation=simple")
        .arg("--resultdir")
        .arg(topdir);
    apply_mock_build_env(&mut command, &package.build_env);
    command.arg("--rebuild").arg(srpm_path);
    run_logged_command_with_file_tails(
        &mut command,
        logger,
        vec![
            ("root.log".to_string(), topdir.join("root.log")),
            ("build.log".to_string(), topdir.join("build.log")),
            ("state.log".to_string(), topdir.join("state.log")),
        ],
    )
    .await
}

async fn rewrite_meson_rust_wraps(package_dir: &Path, logger: &LiveLogSink) -> anyhow::Result<()> {
    let subprojects_dir = package_dir.join("subprojects");
    if !tokio::fs::try_exists(&subprojects_dir).await? {
        logger.line("No subprojects directory detected").await?;
        return Ok(());
    }

    let registry_dir = Path::new("/usr/share/cargo/registry");
    if !tokio::fs::try_exists(registry_dir).await? {
        logger.line("Cargo registry cache not available").await?;
        return Ok(());
    }

    let mut registry_entries = Vec::new();
    let mut registry_read_dir = tokio::fs::read_dir(registry_dir).await?;
    while let Some(entry) = registry_read_dir.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                registry_entries.push(name.to_string());
            }
        }
    }
    registry_entries.sort();

    let mut read_dir = tokio::fs::read_dir(&subprojects_dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if !entry.file_type().await?.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.ends_with("-rs.wrap") {
            continue;
        }

        let Some(stem) = file_name.strip_suffix(".wrap") else {
            continue;
        };
        let Some(crate_prefix) = stem.strip_suffix("-rs") else {
            continue;
        };
        let Some(directory_name) = find_registry_directory(&registry_entries, crate_prefix) else {
            continue;
        };

        let contents = tokio::fs::read_to_string(&path).await?;
        let rewritten = rewrite_wrap_contents(&contents, directory_name)?;

        if rewritten != contents {
            logger
                .line(format!(
                    "Rewriting {} to use directory = {}",
                    path.display(),
                    directory_name
                ))
                .await?;
            tokio::fs::write(&path, rewritten).await?;
        }
    }

    Ok(())
}

fn rewrite_wrap_contents(contents: &str, directory_name: &str) -> anyhow::Result<String> {
    let mut ini = Ini::load_from_str(contents)
        .map_err(|error| anyhow::anyhow!("failed to parse Meson wrap file: {}", error))?;
    let sections = ini
        .iter()
        .filter_map(|(section, _)| section.as_ref().map(ToString::to_string))
        .collect::<Vec<_>>();

    for section in sections {
        if let Some(properties) = ini.section_mut(Some(section.clone())) {
            let source_keys = properties
                .iter()
                .map(|(key, _)| key.to_string())
                .filter(|key| key.starts_with("source_"))
                .collect::<Vec<_>>();
            for key in source_keys {
                properties.remove(&key);
            }
            properties.insert("directory".to_string(), directory_name.to_string());
        }
    }

    let mut rendered = Vec::new();
    ini.write_to(&mut rendered)?;
    Ok(String::from_utf8(rendered)?)
}

fn find_registry_directory<'a>(registry_entries: &'a [String], crate_prefix: &str) -> Option<&'a str> {
    let prefix = format!("{crate_prefix}.");
    registry_entries
        .iter()
        .find(|entry| entry.starts_with(&prefix))
        .map(String::as_str)
}

async fn run_logged_command(command: &mut Command, logger: &LiveLogSink) -> anyhow::Result<()> {
    let std_command = command.as_std();
    let mut command_line = std_command.get_program().to_string_lossy().into_owned();
    for arg in std_command.get_args() {
        command_line.push(' ');
        command_line.push_str(&arg.to_string_lossy());
    }
    logger.line(format!("$ {}", command_line)).await?;

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture command stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture command stderr"))?;

    let stdout_task = tokio::spawn(forward_command_stream(stdout, logger.clone(), false));
    let stderr_task = tokio::spawn(forward_command_stream(stderr, logger.clone(), true));

    let status = child.wait().await?;
    stdout_task.await??;
    stderr_task.await??;

    if status.success() {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "command failed with status {}",
        status
    ))
}

async fn run_logged_command_with_file_tails(
    command: &mut Command,
    logger: &LiveLogSink,
    tailed_files: Vec<(String, PathBuf)>,
) -> anyhow::Result<()> {
    let std_command = command.as_std();
    let mut command_line = std_command.get_program().to_string_lossy().into_owned();
    for arg in std_command.get_args() {
        command_line.push(' ');
        command_line.push_str(&arg.to_string_lossy());
    }
    logger.line(format!("$ {}", command_line)).await?;

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture command stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture command stderr"))?;

    let stdout_task = tokio::spawn(forward_command_stream(stdout, logger.clone(), false));
    let stderr_task = tokio::spawn(forward_command_stream(stderr, logger.clone(), true));
    let (tail_shutdown_tx, tail_shutdown_rx) = oneshot::channel();
    let tail_task = tokio::spawn(tail_named_files_until_exit(
        tailed_files,
        logger.clone(),
        tail_shutdown_rx,
    ));

    let status = child.wait().await?;
    let _ = tail_shutdown_tx.send(());
    stdout_task.await??;
    stderr_task.await??;
    tail_task.await??;

    if status.success() {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "command failed with status {}",
        status
    ))
}

async fn tail_named_files_until_exit(
    files: Vec<(String, PathBuf)>,
    logger: LiveLogSink,
    mut shutdown: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let mut offsets: HashMap<PathBuf, usize> = HashMap::new();
    let mut announced: HashMap<PathBuf, bool> = HashMap::new();

    loop {
        let mut made_progress = false;
        for (name, path) in &files {
            if !tokio::fs::try_exists(path).await.unwrap_or(false) {
                continue;
            }

            let bytes = tokio::fs::read(path).await?;
            let offset = offsets.get(path).copied().unwrap_or(0);
            if bytes.len() <= offset {
                continue;
            }

            if !announced.get(path).copied().unwrap_or(false) {
                logger.line(format!("[streaming {}]", name)).await?;
                announced.insert(path.clone(), true);
            }

            logger.write(false, &bytes[offset..]).await?;
            if !bytes.ends_with(b"\n") {
                logger.line("").await?;
            }
            offsets.insert(path.clone(), bytes.len());
            made_progress = true;
        }

        if shutdown.try_recv().is_ok() {
            break;
        }

        if !made_progress {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
                _ = &mut shutdown => {
                    break;
                }
            }
        }
    }

    for (name, path) in &files {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            continue;
        }
        let bytes = tokio::fs::read(path).await?;
        let offset = offsets.get(path).copied().unwrap_or(0);
        if bytes.len() <= offset {
            continue;
        }
        if !announced.get(path).copied().unwrap_or(false) {
            logger.line(format!("[streaming {}]", name)).await?;
        }
        logger.write(false, &bytes[offset..]).await?;
        if !bytes.ends_with(b"\n") {
            logger.line("").await?;
        }
    }

    Ok(())
}

fn apply_build_env(command: &mut Command, build_env: &[synforge_core::BuildEnvVar]) {
    for entry in build_env {
        command.env(entry.key.trim(), entry.value.as_str());
    }
}

fn apply_mock_build_env(command: &mut Command, build_env: &[synforge_core::BuildEnvVar]) {
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

#[derive(Clone)]
struct LiveLogSink {
    file: Arc<Mutex<tokio::fs::File>>,
    transport: Option<WorkerTransportHandle>,
}

impl LiveLogSink {
    async fn new(path: &Path, transport: Option<WorkerTransportHandle>) -> anyhow::Result<Self> {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .with_context(|| format!("failed to open {}", path.display()))?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            transport,
        })
    }

    async fn section(&self, title: impl AsRef<str>) -> anyhow::Result<()> {
        self.write(false, format!("\n::section::{}\n", title.as_ref()).as_bytes())
            .await
    }

    async fn line(&self, message: impl AsRef<str>) -> anyhow::Result<()> {
        self.write(false, format!("{}\n", message.as_ref()).as_bytes()).await
    }

    async fn write(&self, stderr: bool, bytes: &[u8]) -> anyhow::Result<()> {
        {
            let mut file = self.file.lock().await;
            file.write_all(bytes).await?;
            file.flush().await?;
        }
        if let Some(transport) = &self.transport {
            transport.send_log_chunk(bytes.to_vec()).await?;
        }
        write_console(stderr, bytes).await?;
        Ok(())
    }
}

async fn write_console(stderr: bool, bytes: &[u8]) -> anyhow::Result<()> {
    let buffer = bytes.to_vec();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        if stderr {
            let mut stream = std::io::stderr();
            stream.write_all(&buffer)?;
            stream.flush()?;
        } else {
            let mut stream = std::io::stdout();
            stream.write_all(&buffer)?;
            stream.flush()?;
        }
        Ok(())
    })
    .await??;
    Ok(())
}

#[derive(Clone)]
pub struct WorkerTransportHandle {
    framed: Arc<Mutex<Framed<TcpStream, LengthDelimitedCodec>>>,
}

impl WorkerTransportHandle {
    async fn connect(connect_addr: &str, worker_id: &str) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(connect_addr)
            .await
            .with_context(|| format!("failed to connect worker socket {}", connect_addr))?;
        let framed = Arc::new(Mutex::new(Framed::new(stream, LengthDelimitedCodec::new())));
        let transport = Self { framed };
        transport
            .send_message(WorkerWireMessage::Hello {
                worker_id: worker_id.to_string(),
            })
            .await?;
        Ok(transport)
    }

    async fn receive_assignment(&self) -> anyhow::Result<WorkerJobPayload> {
        match self.read_message().await? {
            WorkerWireMessage::JobAssignment { payload } => Ok(payload),
            WorkerWireMessage::Error { message } => Err(anyhow::anyhow!(message)),
            other => Err(anyhow::anyhow!("unexpected worker assignment message: {:?}", other)),
        }
    }

    async fn send_log_chunk(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.send_message(WorkerWireMessage::LogChunk { bytes }).await
    }

    async fn send_artifact(&self, artifact: &BuildArtifact) -> anyhow::Result<()> {
        let path = artifact.relative_repo_path.to_string_lossy().to_string();
        self.send_message(WorkerWireMessage::ArtifactStart {
            path,
            kind: artifact.kind,
        })
        .await?;
        let bytes = tokio::fs::read(&artifact.path).await?;
        for chunk in bytes.chunks(64 * 1024) {
            self.send_message(WorkerWireMessage::ArtifactChunk {
                bytes: chunk.to_vec(),
            })
            .await?;
        }
        self.send_message(WorkerWireMessage::ArtifactComplete).await
    }

    async fn send_result(&self, result: WorkerResult) -> anyhow::Result<()> {
        self.send_message(WorkerWireMessage::Result { result }).await
    }

    async fn send_message(&self, message: WorkerWireMessage) -> anyhow::Result<()> {
        let bytes = bincode::serialize(&message)?;
        let mut framed = self.framed.lock().await;
        framed.send(Bytes::from(bytes)).await?;
        Ok(())
    }

    async fn read_message(&self) -> anyhow::Result<WorkerWireMessage> {
        let mut framed = self.framed.lock().await;
        let bytes = framed
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("worker socket disconnected"))??;
        Ok(bincode::deserialize(&bytes)?)
    }
}

async fn forward_command_stream<R>(
    mut reader: R,
    logger: LiveLogSink,
    stderr: bool,
) -> anyhow::Result<()>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        logger.write(stderr, &buffer[..read]).await?;
    }
    Ok(())
}

async fn command_exists(name: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {} >/dev/null 2>&1", name))
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
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
    let has_srpm = artifacts.iter().any(|artifact| artifact.kind == ArtifactKind::Srpm);
    if !has_srpm {
        let mut source_artifacts = collect_artifacts(package, source_topdir, mock_chroot)
            .await
            .unwrap_or_default();
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
        arch: artifact_rpm_arch(&path)
            .unwrap_or("unknown")
            .to_string(),
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
