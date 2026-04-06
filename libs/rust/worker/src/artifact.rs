use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use glob::glob;
use sha2::{Digest, Sha256};
use synforge_core::{
    model::{ArtifactKind, BuildArtifact},
    package::PackageDefinition,
};
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
    let bytes = tokio::fs::read(&path)
        .await
        .with_context(|| format!("failed to read artifact {}", path.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let kind = classify_rpm_artifact(&path);
    let artifact_root = topdir.parent().unwrap_or(topdir);
    Ok(BuildArtifact {
        id: Uuid::now_v7(),
        package_name: package.name.clone(),
        mock_chroot: mock_chroot.to_string(),
        size_bytes: bytes.len() as u64,
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use synforge_core::model::ArtifactKind;

    use super::classify_rpm_artifact;

    #[test]
    fn classifies_debug_packages_using_nevra_name() {
        assert_eq!(
            classify_rpm_artifact(Path::new(
                "amneziawg-tools-debugsource-1.0.20260403git5d6179a-1.fc44.x86_64.rpm"
            )),
            ArtifactKind::Debugsource
        );
        assert_eq!(
            classify_rpm_artifact(Path::new(
                "amneziawg-tools-debuginfo-1.0.20260403git5d6179a-1.fc44.x86_64.rpm"
            )),
            ArtifactKind::Debuginfo
        );
        assert_eq!(
            classify_rpm_artifact(Path::new(
                "mesa-libEGL-debuginfo-26.1.0-0.7.20260404.12.8730c03.fc44.x86_64.rpm"
            )),
            ArtifactKind::Debuginfo
        );
        assert_eq!(
            classify_rpm_artifact(Path::new(
                "mesa-debugsource-26.1.0-0.7.20260404.12.8730c03.fc44.x86_64.rpm"
            )),
            ArtifactKind::Debugsource
        );
    }

    #[test]
    fn keeps_standard_rpm_kinds() {
        assert_eq!(
            classify_rpm_artifact(Path::new("mesa-26.1.0-1.fc44.x86_64.rpm")),
            ArtifactKind::Rpm
        );
        assert_eq!(
            classify_rpm_artifact(Path::new("foo-debuginfo-helper-1.0-1.fc44.x86_64.rpm")),
            ArtifactKind::Rpm
        );
        assert_eq!(
            classify_rpm_artifact(Path::new("mesa-26.1.0-1.fc44.src.rpm")),
            ArtifactKind::Srpm
        );
    }
}
