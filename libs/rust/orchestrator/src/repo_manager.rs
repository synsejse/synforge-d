use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::Context;
use synforge_core::{
    config::DaemonConfig,
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, PublishedRepoFile, RepoPublication,
        WorkerBuildResult, format_timestamp, now_utc,
    },
    package::PackageDefinition,
};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct FileRepoManager;

impl FileRepoManager {
    pub async fn ensure_repo(&self, config: &DaemonConfig) -> anyhow::Result<()> {
        regenerate_metadata(config.runtime_paths().repo_dir()).await
    }

    pub async fn publish_build(
        &self,
        package: &PackageDefinition,
        worker_result: &WorkerBuildResult,
        config: &DaemonConfig,
    ) -> anyhow::Result<RepoPublication> {
        let paths = config.runtime_paths();
        if worker_result.status != BuildStatus::Succeeded {
            return Err(anyhow::anyhow!(
                "cannot publish artifacts for failed worker result"
            ));
        }
        let published_at = now_utc();
        let mut files = Vec::new();
        let mut seen_repo_paths = HashSet::new();
        for artifact in &worker_result.artifacts {
            if artifact.kind == ArtifactKind::Srpm && !package.publish_srpm {
                continue;
            }
            let build_root = build_repo_build_dir(config, package, worker_result.job_id, artifact);
            tokio::fs::create_dir_all(&build_root)
                .await
                .with_context(|| format!("failed to create {}", build_root.display()))?;
            let source_path = paths
                .job_artifacts_dir(worker_result.job_id)
                .join(&artifact.path);
            let file_name = artifact.path.file_name().ok_or_else(|| {
                anyhow::anyhow!("artifact path {} has no filename", artifact.path.display())
            })?;
            let destination = build_root.join(file_name);
            let repo_path = destination
                .strip_prefix(paths.repo_dir())
                .unwrap_or(&destination)
                .to_path_buf();
            if !seen_repo_paths.insert(repo_path.clone()) {
                continue;
            }
            if destination.exists() {
                fs::remove_file(&destination)
                    .with_context(|| format!("failed to replace {}", destination.display()))?;
            }
            fs::hard_link(&source_path, &destination).or_else(|_| {
                fs::copy(&source_path, &destination)
                    .map(|_| ())
                    .with_context(|| {
                        format!(
                            "failed to copy artifact {} to {}",
                            source_path.display(),
                            destination.display()
                        )
                    })
            })?;
            files.push(PublishedRepoFile {
                artifact_id: artifact.id,
                job_id: worker_result.job_id,
                package_name: package.name.clone(),
                mock_chroot: artifact.mock_chroot.clone(),
                repo_path,
                sha256: artifact.sha256.clone(),
                size_bytes: artifact.size_bytes,
                kind: artifact.kind,
                published_at,
            });
        }
        regenerate_metadata(paths.repo_dir()).await?;
        Ok(RepoPublication {
            package_name: package.name.clone(),
            repo_root: paths.repo_dir().to_path_buf(),
            published_at,
            files,
        })
    }

    pub async fn remove_build_files(
        &self,
        files: &[PublishedRepoFile],
        config: &DaemonConfig,
    ) -> anyhow::Result<()> {
        let paths = config.runtime_paths();
        for file in files {
            let path = paths.repo_dir().join(&file.repo_path);
            match tokio::fs::remove_file(&path).await {
                Ok(()) => prune_empty_parents(&path, paths.repo_dir()).await?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(anyhow::Error::from(error))
                        .context(format!("failed to remove {}", path.display()));
                }
            }
        }
        regenerate_metadata(paths.repo_dir()).await?;
        Ok(())
    }
}

fn build_repo_build_dir(
    config: &DaemonConfig,
    package: &PackageDefinition,
    job_id: Uuid,
    artifact: &BuildArtifact,
) -> std::path::PathBuf {
    config
        .runtime_paths()
        .repo_dir()
        .join("packages")
        .join(&package.name)
        .join(&artifact.mock_chroot)
        .join("builds")
        .join(job_id.to_string())
}

async fn regenerate_metadata(repo_dir: &Path) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(repo_dir).await?;
    let output = Command::new("createrepo_c")
        .arg("--update")
        .arg(repo_dir)
        .output()
        .await;
    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(anyhow::anyhow!(
            "createrepo_c failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let repodata = repo_dir.join("repodata");
            tokio::fs::create_dir_all(&repodata).await?;
            tokio::fs::write(
                repodata.join("repomd.xml"),
                format!("<repomd generated=\"{}\" />", format_timestamp(now_utc())),
            )
            .await?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

async fn prune_empty_parents(path: &Path, repo_root: &Path) -> anyhow::Result<()> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir == repo_root {
            break;
        }
        match tokio::fs::remove_dir(dir).await {
            Ok(()) => current = dir.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => current = dir.parent(),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
