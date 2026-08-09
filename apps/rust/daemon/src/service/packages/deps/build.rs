use async_trait::async_trait;
use synforge_core::{
    api::{BuildJobResponse, PackageResponse, PackageTargetCcacheStats},
    error::SynforgeError,
    model::{BuildJob, PublishedRepoFile},
    package::PackageDefinition,
};
use synforge_database::JobStore;
use synforge_git_sync::{
    PackageBuildHistoryReader, PackageDefinitionWriter, PackageDeletionJobReader,
    PackageDeletionRunner, PackageLookup,
};
use synforge_worker_host::{
    ActiveTargetBuildReader, BuildJobWriter, BuildQueue, LastSuccessfulRevisionReader,
    QueuedBuildRequest, TargetBuildBackoffReader,
};
use uuid::Uuid;

use super::DaemonPackageDeps;

#[async_trait]
impl BuildQueue for DaemonPackageDeps {
    async fn enqueue_build(&self, build: QueuedBuildRequest) -> anyhow::Result<()> {
        self.queue_build_request(build).await
    }
}

#[async_trait]
impl PackageDeletionRunner for DaemonPackageDeps {
    async fn delete_package_job(&self, job_id: Uuid) -> anyhow::Result<()> {
        let published_files = self.load_published_repo_files_for_job(job_id).await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        self.remove_job_record(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        Ok(())
    }
}

#[async_trait]
impl PackageBuildHistoryReader for DaemonPackageDeps {
    async fn count_package_builds(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<u64> {
        self.load_package_build_count(package_name, include_deleted)
            .await
    }

    async fn list_package_builds(
        &self,
        package_name: &str,
        limit: usize,
        offset: usize,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.load_package_builds(package_name, limit, offset, include_deleted)
            .await
    }

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.load_published_repo_files_for_package(package_name)
            .await
    }

    async fn list_package_ccache_stats(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PackageTargetCcacheStats>> {
        self.load_package_ccache_stats(package_name).await
    }
}

#[async_trait]
impl PackageLookup for DaemonPackageDeps {
    async fn find_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>> {
        self.load_package(package_name).await
    }
}

#[async_trait]
impl PackageDefinitionWriter for DaemonPackageDeps {
    async fn upsert_package_definition(&self, package: &PackageDefinition) -> anyhow::Result<()> {
        self.save_package_definition(package).await
    }
}

#[async_trait]
impl ActiveTargetBuildReader for DaemonPackageDeps {
    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        self.load_has_active_job_for_target(package_name, mock_chroot)
            .await
    }
}

#[async_trait]
impl LastSuccessfulRevisionReader for DaemonPackageDeps {
    async fn get_last_successful_revision(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<String>> {
        self.load_last_successful_revision(package_name, mock_chroot)
            .await
    }
}

#[async_trait]
impl TargetBuildBackoffReader for DaemonPackageDeps {
    async fn get_target_backoff_wait_seconds(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<Option<u64>> {
        self.load_target_backoff_wait_seconds(package_name, mock_chroot)
            .await
    }
}

#[async_trait]
impl BuildJobWriter for DaemonPackageDeps {
    async fn insert_build_job(&self, job: &BuildJob) -> anyhow::Result<bool> {
        self.save_build_job(job).await
    }

    async fn cancel_build_job(&self, job_id: Uuid, message: &str) -> anyhow::Result<()> {
        let _ = self
            .store
            .finish_job(
                job_id,
                synforge_core::model::BuildStatus::Failed,
                Some(message),
                &[],
                &[],
                &[],
                None,
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PackageDeletionJobReader for DaemonPackageDeps {
    async fn list_jobs_for_package(
        &self,
        package_name: &str,
        include_deleted: bool,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.load_jobs_for_package(package_name, include_deleted)
            .await
    }
}
