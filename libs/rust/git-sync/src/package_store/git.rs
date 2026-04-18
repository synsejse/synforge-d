use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Context;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub(super) async fn run_git(
    dir: Option<&Path>,
    args: &[&str],
    timeout: Duration,
) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = tokio::time::timeout(timeout, command.output())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(super) async fn run_git_clone_with_progress<F, Fut>(
    dir: Option<&Path>,
    args: &[&str],
    timeout: Duration,
    mut on_progress: F,
) -> anyhow::Result<()>
where
    F: FnMut(u8, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn git clone command: git {}", args.join(" ")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture git clone stderr"))?;
    let mut stderr_lines = BufReader::new(stderr).lines();
    let mut stderr_output = String::new();

    loop {
        let next_line = tokio::time::timeout(timeout, stderr_lines.next_line())
            .await
            .with_context(|| {
                format!(
                    "git {} timed out after {}s",
                    args.join(" "),
                    timeout.as_secs()
                )
            })??;
        let Some(line) = next_line else {
            break;
        };
        if !stderr_output.is_empty() {
            stderr_output.push('\n');
        }
        stderr_output.push_str(&line);
        if let Some((progress_percent, message)) = parse_git_clone_progress_line(&line) {
            on_progress(progress_percent, message).await;
        }
    }

    let status = tokio::time::timeout(timeout, child.wait())
        .await
        .with_context(|| {
            format!(
                "git {} timed out after {}s",
                args.join(" "),
                timeout.as_secs()
            )
        })??;
    if !status.success() {
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr_output);
    }
    Ok(())
}

fn parse_git_clone_progress_line(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim();
    if trimmed.starts_with("Cloning into") {
        return Some((2, "Starting clone…".to_string()));
    }
    if trimmed.contains("Checking connectivity") {
        return Some((99, "Checking repository connectivity…".to_string()));
    }

    for (marker, stage) in [
        ("Receiving objects:", "Receiving objects"),
        ("Resolving deltas:", "Resolving deltas"),
        ("Compressing objects:", "Compressing objects"),
    ] {
        let Some((_, rest)) = trimmed.split_once(marker) else {
            continue;
        };
        let Some(progress_percent) = extract_progress_percent(rest) else {
            continue;
        };
        return Some((progress_percent, format!("{stage}… {progress_percent}%")));
    }
    None
}

fn extract_progress_percent(segment: &str) -> Option<u8> {
    let percent_index = segment.find('%')?;
    let token = segment[..percent_index].split_whitespace().last()?;
    let digits = token
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u8>().ok().map(|value| value.min(100))
}
