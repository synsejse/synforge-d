use std::sync::Arc;

use synforge_core::{
    config::DaemonConfig,
    model::{WorkerAction, WorkerBuildPayload, WorkerJobPayload},
};
use tracing::instrument;

use crate::job_lifecycle::JobLifecycle;
use crate::scheduler::{BuildScheduler, QueuedBuild};
use crate::workers::DockerWorkerLauncher;

#[derive(Clone)]
pub struct BuildRunner {
    config: DaemonConfig,
    worker_launcher: Arc<DockerWorkerLauncher>,
    lifecycle: Arc<JobLifecycle>,
    scheduler: BuildScheduler,
}

impl BuildRunner {
    pub fn new(
        config: DaemonConfig,
        worker_launcher: Arc<DockerWorkerLauncher>,
        lifecycle: Arc<JobLifecycle>,
        scheduler: BuildScheduler,
    ) -> Self {
        Self {
            config,
            worker_launcher,
            lifecycle,
            scheduler,
        }
    }

    #[instrument(skip(self, build), fields(job_id = %build.job_id, package = %build.package.name, mock_chroot = %build.mock_chroot))]
    pub async fn process_build(&self, build: QueuedBuild) -> anyhow::Result<()> {
        let package_name = build.package.name.clone();
        let mock_chroot = build.mock_chroot.clone();
        let result = self.process_build_inner(build).await;
        self.scheduler.release_target(&package_name, &mock_chroot);
        result
    }

    async fn process_build_inner(&self, build: QueuedBuild) -> anyhow::Result<()> {
        let paths = self.config.runtime_paths();
        let job_root = paths.job_root(build.job_id);
        tokio::fs::create_dir_all(paths.job_artifacts_dir(build.job_id)).await?;

        let payload = WorkerJobPayload {
            job_id: build.job_id,
            workspace_dir: job_root,
            timeout_seconds: build.package.build_timeout_seconds,
            action: WorkerAction::Build(WorkerBuildPayload {
                package_name: build.package.name.clone(),
                package: build.package.clone(),
                mock_chroot: build.mock_chroot.clone(),
                trigger: build.trigger,
                revision: build.revision.comparison_key(),
                source_commit: build.revision.content_digest.clone(),
            }),
        };
        let execution = self.worker_launcher.run_job(&payload, &self.config).await;
        match execution {
            Ok(execution) => self.lifecycle.finalize_execution(&build, execution).await,
            Err(error) => self.lifecycle.fail_launch(&build, &error.to_string()).await,
        }
    }
}
