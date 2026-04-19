use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use tokio::process::Command;
use tokio::sync::oneshot;

use super::BuildLogger;

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

pub(crate) async fn run_mock_command(
    command: &mut Command,
    logger: &BuildLogger,
    mock_result_dir: &std::path::Path,
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
    let mut offsets: std::collections::HashMap<PathBuf, u64> = std::collections::HashMap::new();

    loop {
        let mut made_progress = false;
        for (name, path) in &files {
            if !tokio::fs::try_exists(path).await.unwrap_or(false) {
                continue;
            }
            let offset = offsets.entry(path.clone()).or_insert(0);
            if stream_new_file_bytes(path, offset, name, &logger).await? {
                made_progress = true;
            }
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
        let offset = offsets.entry(path.clone()).or_insert(0);
        stream_new_file_bytes(path, offset, name, &logger).await?;
    }

    Ok(())
}

async fn stream_new_file_bytes(
    path: &std::path::Path,
    offset: &mut u64,
    name: &str,
    logger: &BuildLogger,
) -> anyhow::Result<bool> {
    let mut file = tokio::fs::File::open(path).await?;
    let len = file.metadata().await?.len();
    if len < *offset {
        *offset = 0;
    }
    if len == *offset {
        return Ok(false);
    }

    file.seek(std::io::SeekFrom::Start(*offset)).await?;
    let mut buffer = [0_u8; 8192];
    let mut wrote = false;
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        logger.append_named_log(name, &buffer[..read]).await?;
        *offset += read as u64;
        wrote = true;
    }
    Ok(wrote)
}
