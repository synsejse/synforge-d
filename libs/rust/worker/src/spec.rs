use std::path::Path;

use anyhow::Context;
use synforge_core::{
    model::{WorkerJobPayload, WorkerParsePayload, WorkerParseResult},
    package::{ParsedSpec, SpecRevision},
};
use tokio::process::Command;

use crate::git::{clone_repository, git_rev_parse};

pub(crate) async fn execute_spec_parse(
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
        revision: SpecRevision::new(parsed.version, parsed.release, Some(commit)),
    })
}

pub(crate) async fn query_spec_metadata(spec_path: &Path) -> anyhow::Result<ParsedSpec> {
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
