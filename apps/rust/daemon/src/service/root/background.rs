use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::{Semaphore, mpsc, watch};
use tracing::{error, info, warn};

use crate::service::SynforgeService;
use synforge_worker_host::QueuedBuild;

fn describe_panic(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

impl SynforgeService {
    pub async fn graceful_shutdown(&self) {
        warn!("shutdown requested; stopping active worker containers");
        let _ = self.shutdown_tx.send(true);
        self.task_tracker.close();
        if let Err(error) = self.worker_launcher.shutdown().await {
            error!("failed to stop active worker containers: {}", error);
        }
        if let Err(error) = self
            .lifecycle
            .abort_unfinished_jobs("daemon shutdown requested")
            .await
        {
            error!("failed to abort unfinished jobs during shutdown: {}", error);
        }
        self.task_tracker.wait().await;
    }

    pub(super) fn start_queue_runner(
        self: &Arc<Self>,
        mut queue_rx: mpsc::Receiver<QueuedBuild>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let runner = self.runner.clone();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_builds));
        let task_tracker = self.task_tracker.clone();
        info!(
            max_concurrent_builds = self.config.max_concurrent_builds,
            "starting build queue runner"
        );
        task_tracker.clone().spawn(async move {
            loop {
                let maybe_build = tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    build = queue_rx.recv() => build,
                };
                let Some(build) = maybe_build else {
                    break;
                };
                info!(
                    job_id = %build.job_id,
                    package_name = %build.package.name,
                    mock_chroot = %build.mock_chroot,
                    "build dequeued for execution"
                );
                let runner = runner.clone();
                let semaphore = Arc::clone(&semaphore);
                let task_tracker = task_tracker.clone();
                match semaphore.acquire_owned().await {
                    Ok(permit) => {
                        let job_id = build.job_id;
                        let package_name = build.package.name.clone();
                        let mock_chroot = build.mock_chroot.clone();
                        task_tracker.spawn(async move {
                            let _permit = permit;
                            let outcome = AssertUnwindSafe(runner.process_build(build))
                                .catch_unwind()
                                .await;
                            match outcome {
                                Ok(Ok(())) => {}
                                Ok(Err(error)) => error!(
                                    job_id = %job_id,
                                    package_name = %package_name,
                                    mock_chroot = %mock_chroot,
                                    "build processing failed: {}", error
                                ),
                                Err(payload) => error!(
                                    job_id = %job_id,
                                    package_name = %package_name,
                                    mock_chroot = %mock_chroot,
                                    "build task panicked: {}", describe_panic(payload)
                                ),
                            }
                        });
                    }
                    Err(error) => error!("failed to acquire build semaphore: {}", error),
                }
            }
        });
    }

    pub(super) fn start_poller(self: &Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        let service = Arc::clone(self);
        info!(
            poller_tick_seconds = service.config.poller_tick_seconds,
            "starting poller"
        );
        self.task_tracker.spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(
                service.config.poller_tick_seconds,
            ));
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    _ = ticker.tick() => {
                        if let Err(error) = service.poll_once().await {
                            warn!("polling failed: {}", error);
                        }
                    }
                }
            }
        });
    }
}
