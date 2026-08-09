use std::path::PathBuf;

use super::install_staged_build;

struct SwapFixture {
    _root: tempfile::TempDir,
    final_build_dir: PathBuf,
    staging: tempfile::TempDir,
}

async fn swap_fixture() -> SwapFixture {
    let root = tempfile::tempdir().expect("temporary repository root");
    let final_build_dir = root.path().join("repo/packages/example/builds/job-id");
    tokio::fs::create_dir_all(&final_build_dir)
        .await
        .expect("existing build directory");
    tokio::fs::write(final_build_dir.join("old.rpm"), b"old")
        .await
        .expect("existing build artifact");

    let staging = tempfile::tempdir_in(root.path()).expect("publication staging directory");
    let staged_build_dir = staging.path().join("build");
    tokio::fs::create_dir_all(&staged_build_dir)
        .await
        .expect("staged build directory");
    tokio::fs::write(staged_build_dir.join("new.rpm"), b"new")
        .await
        .expect("staged build artifact");

    SwapFixture {
        _root: root,
        final_build_dir,
        staging,
    }
}

#[tokio::test]
async fn rollback_restores_previous_build_directory() {
    let fixture = swap_fixture().await;
    let installed = install_staged_build(&fixture.final_build_dir, fixture.staging)
        .await
        .expect("install staged build");

    assert!(
        tokio::fs::try_exists(fixture.final_build_dir.join("new.rpm"))
            .await
            .expect("inspect new artifact")
    );
    assert!(
        !tokio::fs::try_exists(fixture.final_build_dir.join("old.rpm"))
            .await
            .expect("inspect old artifact")
    );

    installed.rollback().await.expect("rollback staged build");

    assert!(
        tokio::fs::try_exists(fixture.final_build_dir.join("old.rpm"))
            .await
            .expect("inspect restored artifact")
    );
    assert!(
        !tokio::fs::try_exists(fixture.final_build_dir.join("new.rpm"))
            .await
            .expect("inspect rolled-back artifact")
    );
}

#[tokio::test]
async fn commit_keeps_new_build_directory() {
    let fixture = swap_fixture().await;
    let installed = install_staged_build(&fixture.final_build_dir, fixture.staging)
        .await
        .expect("install staged build");

    installed.commit();

    assert!(
        tokio::fs::try_exists(fixture.final_build_dir.join("new.rpm"))
            .await
            .expect("inspect committed artifact")
    );
    assert!(
        !tokio::fs::try_exists(fixture.final_build_dir.join("old.rpm"))
            .await
            .expect("inspect replaced artifact")
    );
}
