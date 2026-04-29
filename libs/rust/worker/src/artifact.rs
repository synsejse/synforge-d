use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use glob::glob;
use hex::encode as hex_encode;
use sha2::{Digest, Sha256};
use synforge_core::{
    model::{ArtifactKind, BuildArtifact},
    package::PackageDefinition,
};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub(crate) async fn collect_artifacts(
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
                if seen.insert(artifact.file.clone()) {
                    artifacts.push(artifact);
                }
            }
        }
    }
    artifacts.sort_by(|left, right| left.file.cmp(&right.file));
    Ok(artifacts)
}

pub(crate) async fn collect_success_artifacts(
    package: &PackageDefinition,
    mock_topdir: &Path,
    source_topdir: &Path,
    mock_chroot: &str,
) -> anyhow::Result<Vec<BuildArtifact>> {
    let mut artifacts = collect_artifacts(package, mock_topdir, mock_chroot).await?;
    let has_srpm = artifacts
        .iter()
        .any(|artifact| artifact.kind == ArtifactKind::Srpm);
    if !has_srpm {
        let mut source_artifacts = collect_artifacts(package, source_topdir, mock_chroot)
            .await
            .with_context(|| {
                format!(
                    "failed to collect source artifacts from {}",
                    source_topdir.display()
                )
            })?;
        source_artifacts.retain(|artifact| artifact.kind == ArtifactKind::Srpm);
        artifacts.extend(source_artifacts);
        artifacts.sort_by(|left, right| left.file.cmp(&right.file));
    }
    Ok(artifacts)
}

fn classify_rpm_artifact(path: &Path) -> ArtifactKind {
    let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
        return ArtifactKind::Other;
    };

    if filename.ends_with(".src.rpm") {
        ArtifactKind::Srpm
    } else if filename.ends_with(".rpm") {
        match rpm_name_component(filename) {
            Some(name) if name.ends_with("-debuginfo") => ArtifactKind::Debuginfo,
            Some(name) if name.ends_with("-debugsource") => ArtifactKind::Debugsource,
            _ => ArtifactKind::Rpm,
        }
    } else {
        ArtifactKind::Other
    }
}

fn rpm_name_component(filename: &str) -> Option<&str> {
    let nvra = filename.strip_suffix(".rpm")?;
    let (name_version_release, _arch) = nvra.rsplit_once('.')?;
    let (name_version, _release) = name_version_release.rsplit_once('-')?;
    let (name, _version) = name_version.rsplit_once('-')?;
    Some(name)
}

async fn build_artifact(
    package: &PackageDefinition,
    topdir: &Path,
    path: PathBuf,
    mock_chroot: &str,
) -> anyhow::Result<BuildArtifact> {
    let mut file = tokio::fs::File::open(&path)
        .await
        .with_context(|| format!("failed to open artifact {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size_bytes += read as u64;
    }
    let sha256 = hex_encode(hasher.finalize());
    let kind = classify_rpm_artifact(&path);
    let artifact_root = topdir.parent().unwrap_or(topdir);
    Ok(BuildArtifact {
        id: Uuid::now_v7(),
        package_name: package.name.clone(),
        mock_chroot: mock_chroot.to_string(),
        size_bytes,
        file: path.file_name().map(PathBuf::from).unwrap_or_else(|| {
            path.strip_prefix(artifact_root)
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("artifact.rpm"))
        }),
        sha256,
        kind,
        signing_status: None,
        signing_error_message: None,
    })
}
