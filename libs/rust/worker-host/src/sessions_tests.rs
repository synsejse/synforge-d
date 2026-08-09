use std::path::PathBuf;

use synforge_core::{
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, BuildTrigger, WorkerAction, WorkerBuildPayload,
        WorkerBuildResult, WorkerJobPayload, WorkerResult,
    },
    package::{PackageDefinition, SpecSource},
};
use tempfile::tempdir;
use uuid::Uuid;

use super::{WorkerSessionBroker, merge_result};

fn package() -> PackageDefinition {
    PackageDefinition {
        name: "XYZ".to_string(),
        description: String::new(),
        enabled: true,
        repo_subdir: "XYZ".to_string(),
        publish_srpm: true,
        publish_debuginfo: true,
        network_access: false,
        mock_chroots: vec!["fedora-42-x86_64".to_string()],
        source: SpecSource {
            repo_url: "https://example.com/package.git".to_string(),
            spec_file: "package.spec".to_string(),
            poll: true,
        },
        poll_interval_seconds: 900,
        build_timeout_seconds: 7200,
        package_history_count: 3,
        cpu_limit_millicores: None,
        memory_limit_mb: None,
        ccache_enabled: false,
        ccache_max_size_mb: None,
        build_env: Vec::new(),
        spec_file: PathBuf::from("package.spec"),
        version: "1.0".to_string(),
        release: "1".to_string(),
    }
}

fn payload(job_id: Uuid) -> WorkerJobPayload {
    WorkerJobPayload {
        job_id,
        workspace_dir: PathBuf::from("/workspace"),
        timeout_seconds: 60,
        action: WorkerAction::Build(Box::new(WorkerBuildPayload {
            package_name: "XYZ".to_string(),
            package: package(),
            mock_chroot: "fedora-42-x86_64".to_string(),
            trigger: BuildTrigger::Api,
            revision: "1.0-1".to_string(),
            checkout_commit: None,
        })),
    }
}

fn artifact(id: Uuid, file: &str) -> BuildArtifact {
    BuildArtifact {
        id,
        package_name: "XYZ".to_string(),
        mock_chroot: "fedora-42-x86_64".to_string(),
        file: PathBuf::from(file),
        sha256: "daemon-computed".to_string(),
        size_bytes: 42,
        kind: ArtifactKind::Rpm,
        signing_status: None,
        signing_error_message: None,
    }
}

#[test]
fn merge_result_replaces_worker_owned_artifact_identity() {
    let job_id = Uuid::now_v7();
    let uploaded = artifact(Uuid::now_v7(), "package.rpm");
    let worker_supplied = artifact(Uuid::now_v7(), "/tmp/escape.rpm");
    let result = WorkerResult::Build(WorkerBuildResult {
        job_id: Uuid::now_v7(),
        package_name: "different-package".to_string(),
        status: BuildStatus::Succeeded,
        artifacts: vec![worker_supplied],
        message: None,
    });

    let merged = merge_result(
        job_id,
        &payload(job_id),
        result,
        std::slice::from_ref(&uploaded),
    )
    .expect("valid terminal result");
    let WorkerResult::Build(merged) = merged else {
        panic!("expected build result");
    };
    assert_eq!(merged.job_id, job_id);
    assert_eq!(merged.package_name, "XYZ");
    assert_eq!(merged.artifacts, vec![uploaded]);
}

#[test]
fn merge_result_rejects_non_terminal_status() {
    let job_id = Uuid::now_v7();
    let result = WorkerResult::Build(WorkerBuildResult {
        job_id,
        package_name: "XYZ".to_string(),
        status: BuildStatus::Running,
        artifacts: Vec::new(),
        message: None,
    });

    assert!(merge_result(job_id, &payload(job_id), result, &[]).is_err());
}

#[tokio::test]
async fn artifact_storage_path_is_pinned_to_job_boundary() {
    let root = tempdir().expect("temporary session root");
    let broker = WorkerSessionBroker::new(root.path().to_path_buf());
    let job_id = Uuid::now_v7();
    broker
        .create_session(job_id, payload(job_id))
        .await
        .expect("create worker session");

    let (_, path) = broker
        .artifact_storage_path(job_id, "package.rpm")
        .expect("validated storage path");
    assert!(path.strictpath_ends_with("fedora-42-x86_64/package.rpm"));
    assert!(
        broker
            .artifact_storage_path(job_id, "../escape.rpm")
            .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn artifact_storage_path_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = tempdir().expect("temporary session root");
    let outside = tempdir().expect("temporary outside root");
    let broker = WorkerSessionBroker::new(root.path().to_path_buf());
    let job_id = Uuid::now_v7();
    broker
        .create_session(job_id, payload(job_id))
        .await
        .expect("create worker session");
    symlink(
        outside.path(),
        broker.job_root(job_id).join("artifacts/fedora-42-x86_64"),
    )
    .expect("create escaping symlink");

    assert!(broker.artifact_storage_path(job_id, "package.rpm").is_err());
}
