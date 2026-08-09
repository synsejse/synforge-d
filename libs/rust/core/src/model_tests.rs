use std::path::PathBuf;

use uuid::Uuid;

use super::{ArtifactKind, BuildArtifact};

fn artifact(mock_chroot: &str, file: &str) -> BuildArtifact {
    BuildArtifact {
        id: Uuid::nil(),
        package_name: "XYZ".to_string(),
        mock_chroot: mock_chroot.to_string(),
        file: PathBuf::from(file),
        sha256: String::new(),
        size_bytes: 0,
        kind: ArtifactKind::Rpm,
        signing_status: None,
        signing_error_message: None,
    }
}

#[test]
fn artifact_storage_path_is_derived_from_validated_components() {
    assert_eq!(
        artifact("fedora-42-x86_64", "package-1.0-1.x86_64.rpm")
            .storage_path()
            .expect("valid artifact path"),
        PathBuf::from("fedora-42-x86_64/package-1.0-1.x86_64.rpm")
    );
}

#[test]
fn artifact_storage_path_rejects_untrusted_path_structure() {
    for file in ["../escape.rpm", "/tmp/escape.rpm", "nested/file.rpm"] {
        assert!(
            artifact("fedora-42-x86_64", file).storage_path().is_err(),
            "accepted {file:?}"
        );
    }
    assert!(
        artifact("..-42-x86_64", "package.rpm")
            .storage_path()
            .is_err()
    );
}

#[test]
fn artifact_kind_is_derived_from_rpm_filename() {
    assert_eq!(
        ArtifactKind::from_file_name("package-1.0-1.x86_64.rpm"),
        ArtifactKind::Rpm
    );
    assert_eq!(
        ArtifactKind::from_file_name("package-1.0-1.src.rpm"),
        ArtifactKind::Srpm
    );
    assert_eq!(
        ArtifactKind::from_file_name("package-debuginfo-1.0-1.x86_64.rpm"),
        ArtifactKind::Debuginfo
    );
    assert_eq!(
        ArtifactKind::from_file_name("package-debugsource-1.0-1.x86_64.rpm"),
        ArtifactKind::Debugsource
    );
    assert_eq!(
        ArtifactKind::from_file_name("package.log"),
        ArtifactKind::Other
    );
}
