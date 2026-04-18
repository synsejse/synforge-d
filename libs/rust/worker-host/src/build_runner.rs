use std::sync::Arc;

use synforge_core::{
    config::DaemonConfig,
    constants::DEFAULT_WORKER_WORKSPACE_ROOT,
    model::{WorkerAction, WorkerBuildPayload, WorkerJobPayload, WorkerResult},
};
use tracing::{error, info, instrument};

use crate::{DockerWorkerLauncher, JobLifecycle, QueuedBuild};

#[derive(Clone)]
pub struct BuildRunner {
    config: DaemonConfig,
    worker_launcher: Arc<DockerWorkerLauncher>,
    lifecycle: Arc<JobLifecycle>,
}

impl BuildRunner {
    pub fn new(
        config: DaemonConfig,
        worker_launcher: Arc<DockerWorkerLauncher>,
        lifecycle: Arc<JobLifecycle>,
    ) -> Self {
        Self {
            config,
            worker_launcher,
            lifecycle,
        }
    }

    #[instrument(skip(self, build), fields(job_id = %build.job_id, package = %build.package.name, mock_chroot = %build.mock_chroot))]
    pub async fn process_build(&self, build: QueuedBuild) -> anyhow::Result<()> {
        self.process_build_inner(build).await
    }

    async fn process_build_inner(&self, build: QueuedBuild) -> anyhow::Result<()> {
        if !self.lifecycle.should_run_job(build.job_id).await? {
            info!(
                job_id = %build.job_id,
                package_name = %build.package.name,
                mock_chroot = %build.mock_chroot,
                "skipping queued build because job is no longer pending"
            );
            return Ok(());
        }
        info!(
            job_id = %build.job_id,
            package_name = %build.package.name,
            mock_chroot = %build.mock_chroot,
            trigger = ?build.trigger,
            "starting queued build"
        );
        let worker_workspace =
            std::path::Path::new(DEFAULT_WORKER_WORKSPACE_ROOT).join(build.job_id.to_string());

        let payload = WorkerJobPayload {
            job_id: build.job_id,
            workspace_dir: worker_workspace,
            timeout_seconds: build.package.build_timeout_seconds,
            action: WorkerAction::Build(Box::new(WorkerBuildPayload {
                package_name: build.package.name.clone(),
                package: build.package.clone(),
                mock_chroot: build.mock_chroot.clone(),
                trigger: build.trigger,
                revision: build.revision.comparison_key(),
                checkout_commit: build.revision.source_commit.clone(),
            })),
        };
        let execution = self.worker_launcher.run_job(&payload, &self.config).await;
        let result = match execution {
            Ok(execution) => {
                if let WorkerResult::Build(build_result) = &execution {
                    info!(
                        job_id = %build.job_id,
                        package_name = %build.package.name,
                        mock_chroot = %build.mock_chroot,
                        status = ?build_result.status,
                        artifact_count = build_result.artifacts.len(),
                        "worker execution finished"
                    );
                }
                self.lifecycle.finalize_execution(&build, execution).await
            }
            Err(error) => {
                error!(
                    job_id = %build.job_id,
                    package_name = %build.package.name,
                    mock_chroot = %build.mock_chroot,
                    error = %error,
                    "worker launch or execution failed"
                );
                self.lifecycle.fail_launch(&build, &error.to_string()).await
            }
        };
        self.worker_launcher.cleanup_session(build.job_id);
        result
    }
}
