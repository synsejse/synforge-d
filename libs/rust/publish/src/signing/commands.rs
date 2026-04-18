use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub(super) fn parse_first_fingerprint(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let mut fields = line.split(':');
        if fields.next() != Some("fpr") {
            return None;
        }
        let fingerprint = fields.nth(8)?;
        if fingerprint.is_empty() {
            None
        } else {
            Some(fingerprint.to_string())
        }
    })
}

pub(super) async fn run_gpg(keyring_dir: &Path, args: &[&str]) -> anyhow::Result<()> {
    let output = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .output()
        .await
        .context("failed to execute gpg")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(super) async fn run_gpg_with_input(
    keyring_dir: &Path,
    args: &[&str],
    input: &[u8],
) -> anyhow::Result<()> {
    let mut child = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn gpg")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .await
            .context("failed to write gpg stdin")?;
    }
    let output = child
        .wait_with_output()
        .await
        .context("failed to wait for gpg process")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(super) async fn run_gpg_capture_stdout(
    keyring_dir: &Path,
    args: &[&str],
) -> anyhow::Result<String> {
    let output = Command::new("gpg")
        .arg("--homedir")
        .arg(keyring_dir)
        .args(args)
        .output()
        .await
        .context("failed to execute gpg")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "gpg command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).context("gpg output was not valid utf-8")
}

pub(super) async fn run_rpm_addsign(
    keyring_dir: &Path,
    key_id: &str,
    artifact_path: &Path,
) -> anyhow::Result<()> {
    let define_gpg_name = format!("_gpg_name {}", key_id);
    let define_gpg_path = format!("_gpg_path {}", keyring_dir.display());
    let output = match run_rpm_resign_command(
        "rpmsign",
        define_gpg_name.as_str(),
        define_gpg_path.as_str(),
        artifact_path,
    )
    .await
    {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => run_rpm_resign_command(
            "rpm",
            define_gpg_name.as_str(),
            define_gpg_path.as_str(),
            artifact_path,
        )
        .await
        .context("failed to execute rpmsign/rpm --resign")?,
        Err(error) => return Err(error).context("failed to execute rpmsign --resign"),
    };

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "rpm signing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

async fn run_rpm_resign_command(
    binary: &str,
    define_gpg_name: &str,
    define_gpg_path: &str,
    artifact_path: &Path,
) -> std::io::Result<std::process::Output> {
    Command::new(binary)
        .arg("--define")
        .arg(define_gpg_name)
        .arg("--define")
        .arg(define_gpg_path)
        .arg("--define")
        .arg("_signature gpg")
        .arg("--define")
        .arg("__gpg /usr/bin/gpg")
        .arg("--resign")
        .arg(artifact_path)
        .output()
        .await
}

pub(super) async fn run_rpm_delsign(artifact_path: &Path) -> anyhow::Result<()> {
    let output = Command::new("rpm")
        .arg("--delsign")
        .arg(artifact_path)
        .output()
        .await
        .context("failed to execute rpm --delsign")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "rpm --delsign failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(super) async fn recompute_artifact_metadata(path: &Path) -> anyhow::Result<(String, u64)> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read artifact {}", path.display()))?;
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((sha256, bytes.len() as u64))
}

#[cfg(unix)]
pub(super) fn set_owner_only_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to set permissions on {}", path.display()))
}

#[cfg(not(unix))]
pub(super) fn set_owner_only_permissions(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_first_fingerprint;

    #[test]
    fn extracts_first_fingerprint_from_colon_output() {
        let output = "sec:-:255:22:ABCDEF1234:1710000000:0::::::scESC:::+:::23::0:\n\
fpr:::::::::0123456789ABCDEF0123456789ABCDEF01234567:\n\
uid:-::::1710000000::ABCDEF1234567890::Example <example@example.com>::::::::::0:\n";
        assert_eq!(
            parse_first_fingerprint(output),
            Some("0123456789ABCDEF0123456789ABCDEF01234567".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_fingerprint_exists() {
        let output = "sec:-:255:22:ABCDEF1234:1710000000:0::::::scESC:::+:::23::0:\n";
        assert_eq!(parse_first_fingerprint(output), None);
    }
}
