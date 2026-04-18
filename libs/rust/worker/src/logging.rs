#[path = "logging/commands.rs"]
mod commands;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::warn;

use crate::protocol::WorkerTransportHandle;

pub(crate) use commands::{command_exists, run_logged_command, run_mock_command};

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
