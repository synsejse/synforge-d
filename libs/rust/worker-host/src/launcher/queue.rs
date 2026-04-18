use async_trait::async_trait;

use crate::{BuildQueue, QueuedBuild, QueuedBuildRequest, WorkerBuildQueue};

#[async_trait]
impl BuildQueue for WorkerBuildQueue {
    async fn enqueue_build(&self, build: QueuedBuildRequest) -> anyhow::Result<()> {
        self.queue_tx
            .send(QueuedBuild {
                package: build.package,
                mock_chroot: build.mock_chroot,
                revision: build.revision,
                trigger: build.trigger,
                job_id: build.job_id,
            })
            .await
            .map_err(|error| anyhow::anyhow!("failed to queue build: {}", error))
    }
}
