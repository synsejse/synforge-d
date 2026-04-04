use std::path::Path;

use anyhow::Context;
use sha2::{Digest, Sha256};
use synforge_core::{
    model::{WorkerJobPayload, WorkerParsePayload, WorkerParseResult},
    package::{ParsedSpec, SpecRevision},
};
use tokio::process::Command;

use crate::git::{clone_repository, git_head_commit};

const RPMSPEC_QUERY_FORMAT: &str = "%{name}\n%{version}\n%{release}\n%{summary}\n";

pub(crate) async fn execute_spec_parse(
    job_payload: &WorkerJobPayload,
    payload: &WorkerParsePayload,
) -> anyhow::Result<WorkerParseResult> {
    let repo_dir = job_payload.workspace_dir.join("repo");
    clone_repository(&payload.source, &repo_dir, None).await?;
    let spec_file = repo_dir.join(&payload.source.spec_file);
    if !tokio::fs::try_exists(&spec_file).await? {
        anyhow::bail!(
            "spec file {} does not exist in repository {}",
            payload.source.spec_file,
            payload.source.repo_url
        );
    }
    let parsed = query_spec_metadata(&spec_file).await?;
    let content_digest = hash_tracked_spec(&spec_file).await?;
    let source_commit = git_head_commit(&repo_dir).await?;
    Ok(WorkerParseResult {
        parsed: parsed.clone(),
        revision: SpecRevision::new(
            parsed.version,
            parsed.release,
            Some(content_digest),
            Some(source_commit),
        ),
    })
}

pub(crate) async fn query_spec_metadata(spec_path: &Path) -> anyhow::Result<ParsedSpec> {
    let output = Command::new("rpmspec")
        .arg("-q")
        .arg("--srpm")
        .arg("--qf")
        .arg(RPMSPEC_QUERY_FORMAT)
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

async fn hash_tracked_spec(spec_path: &Path) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(spec_path)
        .await
        .with_context(|| format!("failed to read tracked spec {}", spec_path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}
