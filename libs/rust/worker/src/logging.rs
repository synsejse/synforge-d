use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::Context;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::{Mutex, oneshot};
use tracing::warn;

use crate::protocol::WorkerTransportHandle;

/// Multi-stream build logger that writes to separate files for different log sources.
/// This avoids interleaving mock output with structured worker logs.
#[derive(Clone)]
pub(crate) struct BuildLogger {
    /// Primary structured log (sections, status messages)
    primary: LogFile,
    /// Transport for streaming to daemon
    transport: Option<WorkerTransportHandle>,
    /// Logs directory for named log streams.
    logs_dir: PathBuf,
}

#[derive(Clone)]
struct LogFile {
    #[allow(dead_code)] // will be used for log path queries
    path: PathBuf,
    file: Arc<Mutex<tokio::fs::File>>,
}

impl LogFile {
    async fn new(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .with_context(|| format!("failed to open {}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn write(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let mut file = self.file.lock().await;
        file.write_all(bytes).await?;
        file.flush().await?;
        Ok(())
    }
}

impl BuildLogger {
    pub(crate) async fn new(
        logs_dir: &Path,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<Self> {
        tokio::fs::create_dir_all(logs_dir).await?;

        let primary = LogFile::new(&logs_dir.join("worker.log")).await?;

        Ok(Self {
            primary,
            transport,
            logs_dir: logs_dir.to_path_buf(),
        })
    }

    /// Write a plain text section header to the primary log.
    pub(crate) async fn section(&self, title: impl AsRef<str>) -> anyhow::Result<()> {
        let message = format!("\n== {} ==\n", title.as_ref());
        self.primary.write(message.as_bytes()).await?;
        self.stream_to_transport("worker.log", message.as_bytes())
            .await?;
        Ok(())
    }

    /// Write a line to the primary log
    pub(crate) async fn line(&self, message: impl AsRef<str>) -> anyhow::Result<()> {
        let formatted = format!("{}\n", message.as_ref());
        self.primary.write(formatted.as_bytes()).await?;
        self.stream_to_transport("worker.log", formatted.as_bytes())
            .await?;
        Ok(())
    }

    /// Write raw bytes to the primary log
    pub(crate) async fn write(&self, bytes: &[u8]) -> anyhow::Result<()> {
        self.primary.write(bytes).await?;
        self.stream_to_transport("worker.log", bytes).await?;
        Ok(())
    }

    /// Get the path to the primary log file
    #[allow(dead_code)] // will be used when we need individual log path
    pub(crate) fn primary_log_path(&self) -> &Path {
        &self.primary.path
    }

    async fn append_named_log(&self, path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let file_path = self.logs_dir.join(path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await
            .with_context(|| format!("failed to open {}", file_path.display()))?;
        file.write_all(bytes).await?;
        file.flush().await?;
        self.stream_to_transport(path, bytes).await?;
        Ok(())
    }

    async fn stream_to_transport(&self, path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        if let Some(transport) = &self.transport {
            // Don't fail the build if transport fails - logs are on disk
            if let Err(e) = transport.send_log_chunk(path, bytes.to_vec()).await {
                warn!("failed to stream log chunk: {}", e);
            }
        }
        Ok(())
    }
}

pub(crate) async fn log_best_effort(logger: &BuildLogger, section: &str, message: &str) {
    if let Err(error) = logger.section(section).await {
        warn!("failed to write worker log section: {}", error);
    }
    if let Err(error) = logger.line(message).await {
        warn!("failed to write worker log line: {}", error);
    }
}

pub(crate) async fn run_logged_command(
    command: &mut Command,
    logger: &BuildLogger,
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

    let status = child.wait().await?;
    stdout_task.await??;
    stderr_task.await??;

    if status.success() {
        return Ok(());
    }
    Err(anyhow::anyhow!("command failed with status {}", status))
}

/// Run a mock command while tailing mock's per-phase log files into separate
/// streamed outputs.
pub(crate) async fn run_mock_command(
    command: &mut Command,
    logger: &BuildLogger,
    mock_result_dir: &Path,
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

    let stdout_task = tokio::spawn(forward_to_console(stdout, false));
    let stderr_task = tokio::spawn(forward_to_console(stderr, true));
    let (tail_shutdown_tx, tail_shutdown_rx) = oneshot::channel();
    let tail_task = tokio::spawn(tail_named_files_until_exit(
        vec![
            (
                "mock-root.log".to_string(),
                mock_result_dir.join("root.log"),
            ),
            (
                "mock-build.log".to_string(),
                mock_result_dir.join("build.log"),
            ),
            (
                "mock-state.log".to_string(),
                mock_result_dir.join("state.log"),
            ),
        ],
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
    Err(anyhow::anyhow!("command failed with status {}", status))
}

pub(crate) async fn command_exists(name: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {} >/dev/null 2>&1", name))
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
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

async fn forward_command_stream<R>(
    mut reader: R,
    logger: BuildLogger,
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
        // Non-mock commands still stream into the primary worker log.
        logger.write(&buffer[..read]).await?;
        write_console(stderr, &buffer[..read]).await?;
    }
    Ok(())
}

async fn forward_to_console<R>(mut reader: R, stderr: bool) -> anyhow::Result<()>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        write_console(stderr, &buffer[..read]).await?;
    }
    Ok(())
}

async fn tail_named_files_until_exit(
    files: Vec<(String, PathBuf)>,
    logger: BuildLogger,
    mut shutdown: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let mut offsets: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();

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

            logger.append_named_log(name, &bytes[offset..]).await?;
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
        logger.append_named_log(name, &bytes[offset..]).await?;
    }

    Ok(())
}
