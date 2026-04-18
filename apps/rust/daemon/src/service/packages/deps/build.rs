use async_trait::async_trait;
use synforge_core::{
    api::{BuildJobResponse, PackageResponse},
    error::SynforgeError,
    model::{BuildJob, PublishedRepoFile},
    package::PackageDefinition,
};
use synforge_git_sync::{
    EnabledPackageCatalog, PackageBuildHistoryReader, PackageDefinitionWriter,
    PackageDeletionJobReader, PackageDeletionRunner, PackageLookup,
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
        self.build_queue.enqueue_build(build).await
    }
}

#[async_trait]
impl PackageDeletionRunner for DaemonPackageDeps {
    async fn delete_package_job(&self, job_id: Uuid) -> anyhow::Result<()> {
        let published_files = self
            .package_store
            .list_published_repo_files_for_job(job_id)
            .await?;
        self.lifecycle
            .remove_published_files(&published_files)
            .await?;
        self.object_storage.delete_job_outputs(job_id).await?;
        self.package_store
            .delete_job(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!(SynforgeError::NotFound(job_id.to_string())))?;
        Ok(())
    }
}

#[async_trait]
impl EnabledPackageCatalog for DaemonPackageDeps {
    async fn list_all_enabled_package_names(&self) -> anyhow::Result<Vec<String>> {
        self.package_store.list_all_enabled_package_names().await
    }
}

#[async_trait]
impl PackageBuildHistoryReader for DaemonPackageDeps {
    async fn count_package_builds(&self, package_name: &str) -> anyhow::Result<u64> {
        self.package_store.count_package_builds(package_name).await
    }

    async fn list_package_builds(
        &self,
        package_name: &str,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.package_store
            .list_package_builds(package_name, limit, offset)
            .await
    }

    async fn list_published_repo_files_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<PublishedRepoFile>> {
        self.package_store
            .list_published_repo_files_for_package(package_name)
            .await
    }
}

#[async_trait]
impl PackageLookup for DaemonPackageDeps {
    async fn find_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>> {
        self.package_store.find_package(package_name).await
    }
}

#[async_trait]
impl PackageDefinitionWriter for DaemonPackageDeps {
    async fn upsert_package_definition(&self, package: &PackageDefinition) -> anyhow::Result<()> {
        self.package_store.upsert_package_definition(package).await
    }
}

#[async_trait]
impl ActiveTargetBuildReader for DaemonPackageDeps {
    async fn has_active_job_for_target(
        &self,
        package_name: &str,
        mock_chroot: &str,
    ) -> anyhow::Result<bool> {
        self.package_store
            .has_active_job_for_target(package_name, mock_chroot)
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
        self.package_store
            .get_last_successful_revision(package_name, mock_chroot)
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
        self.package_store
            .get_target_backoff_wait_seconds(package_name, mock_chroot)
            .await
    }
}

#[async_trait]
impl BuildJobWriter for DaemonPackageDeps {
    async fn insert_build_job(&self, job: &BuildJob) -> anyhow::Result<()> {
        self.package_store.insert_build_job(job).await
    }
}

#[async_trait]
impl PackageDeletionJobReader for DaemonPackageDeps {
    async fn list_jobs_for_package(
        &self,
        package_name: &str,
    ) -> anyhow::Result<Vec<BuildJobResponse>> {
        self.package_store.list_jobs_for_package(package_name).await
    }
}
