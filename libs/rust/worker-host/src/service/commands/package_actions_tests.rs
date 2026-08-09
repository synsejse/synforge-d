use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use synforge_core::{
    api::PackageActionDisposition,
    model::{BuildJob, BuildStatus, BuildTrigger, now_utc},
    package::{PackageDefinition, SpecRevision, SpecSource},
};
use uuid::Uuid;

use super::{PackageActionPlan, PlannedBuild, reserve_and_enqueue_builds};
use crate::service::{BuildJobWriter, BuildQueue, QueuedBuildRequest};

struct FakeBuildDeps {
    reserve: bool,
    fail_queue: bool,
    queued: AtomicUsize,
    cancelled: AtomicUsize,
}

#[async_trait]
impl BuildJobWriter for FakeBuildDeps {
    async fn insert_build_job(&self, _job: &BuildJob) -> anyhow::Result<bool> {
        Ok(self.reserve)
    }

    async fn cancel_build_job(&self, _job_id: Uuid, _message: &str) -> anyhow::Result<()> {
        self.cancelled.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[async_trait]
impl BuildQueue for FakeBuildDeps {
    async fn enqueue_build(&self, _build: QueuedBuildRequest) -> anyhow::Result<()> {
        self.queued.fetch_add(1, Ordering::Relaxed);
        if self.fail_queue {
            anyhow::bail!("queue unavailable");
        }
        Ok(())
    }
}

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

fn plan() -> (PackageActionPlan, Uuid) {
    let job_id = Uuid::now_v7();
    let now = now_utc();
    let package = package();
    let job = BuildJob {
        id: job_id,
        package_name: package.name.clone(),
        mock_chroot: "fedora-42-x86_64".to_string(),
        revision: "1.0-1".to_string(),
        trigger: BuildTrigger::Api,
        status: BuildStatus::Pending,
        sync_operation_id: None,
        spec_file: package.spec_file.clone(),
        worker_container_id: None,
        created_at: now,
        updated_at: now,
        started_at: None,
        finished_at: None,
        signed_at: None,
        error_message: None,
        deleted_at: None,
    };
    let queued = QueuedBuildRequest {
        package,
        mock_chroot: job.mock_chroot.clone(),
        revision: SpecRevision::new("1.0", "1", None, None),
        trigger: job.trigger,
        job_id,
    };
    (
        PackageActionPlan {
            package_name: "XYZ".to_string(),
            trigger: BuildTrigger::Api,
            builds: vec![PlannedBuild { job, queued }],
            results: Vec::new(),
        },
        job_id,
    )
}

#[tokio::test]
async fn reservation_loser_is_blocked_without_queueing() {
    let deps = FakeBuildDeps {
        reserve: false,
        fail_queue: false,
        queued: AtomicUsize::new(0),
        cancelled: AtomicUsize::new(0),
    };
    let (plan, _) = plan();

    let plan = reserve_and_enqueue_builds(&deps, plan)
        .await
        .expect("blocked reservation is a normal result");
    assert_eq!(plan.results.len(), 1);
    assert_eq!(
        plan.results[0].disposition,
        PackageActionDisposition::Blocked
    );
    assert_eq!(plan.results[0].job_id, None);
    assert_eq!(deps.queued.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn reservation_winner_is_queued() {
    let deps = FakeBuildDeps {
        reserve: true,
        fail_queue: false,
        queued: AtomicUsize::new(0),
        cancelled: AtomicUsize::new(0),
    };
    let (plan, job_id) = plan();

    let plan = reserve_and_enqueue_builds(&deps, plan)
        .await
        .expect("queue reserved build");
    assert_eq!(
        plan.results[0].disposition,
        PackageActionDisposition::Queued
    );
    assert_eq!(plan.results[0].job_id, Some(job_id));
    assert_eq!(deps.queued.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn queue_failure_cancels_reservation() {
    let deps = FakeBuildDeps {
        reserve: true,
        fail_queue: true,
        queued: AtomicUsize::new(0),
        cancelled: AtomicUsize::new(0),
    };
    let (plan, _) = plan();

    assert!(reserve_and_enqueue_builds(&deps, plan).await.is_err());
    assert_eq!(deps.cancelled.load(Ordering::Relaxed), 1);
}
