use std::path::{Component, Path};
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use futures_util::StreamExt;
use object_store::{ObjectStore, PutPayload, aws::AmazonS3Builder, path::Path as ObjectPath};
use synforge_core::config::{DaemonConfig, ObjectStorageConfig};
use tracing::debug;
use uuid::Uuid;

use crate::{RepoFileStorage, RuntimeRepoAdapter};

#[async_trait]
pub trait WorkerOutputStorage: Send + Sync {
    async fn store_job_artifact(
        &self,
        job_id: Uuid,
        storage_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()>;

    async fn store_job_log(
        &self,
        job_id: Uuid,
        relative_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct JobObjectStorage {
    store: Option<Arc<dyn ObjectStore>>,
}

impl JobObjectStorage {
    pub async fn from_config(config: &DaemonConfig) -> anyhow::Result<Self> {
        let Some(object_storage) = config.object_storage.as_ref() else {
            return Ok(Self::default());
        };
        let store = build_s3_store(object_storage)?;
        Ok(Self {
            store: Some(Arc::new(store)),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.store.is_some()
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        let mut listing = store.list(Some(&ObjectPath::from("jobs")));
        while let Some(result) = listing.next().await {
            result?;
            break;
        }
        Ok(())
    }

    pub async fn store_job_artifact(
        &self,
        job_id: Uuid,
        storage_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        self.store_local_file(self.artifact_key(job_id, storage_path), local_path)
            .await
    }

    pub async fn store_job_log(
        &self,
        job_id: Uuid,
        relative_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        self.store_local_file(self.log_key(job_id, relative_path), local_path)
            .await
    }

    pub async fn restore_job_artifact(
        &self,
        job_id: Uuid,
        storage_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<bool> {
        self.restore_to_local_path(self.artifact_key(job_id, storage_path), local_path)
            .await
    }

    pub async fn restore_job_log(
        &self,
        job_id: Uuid,
        relative_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<bool> {
        self.restore_to_local_path(self.log_key(job_id, relative_path), local_path)
            .await
    }

    pub async fn delete_job_outputs(&self, job_id: Uuid) -> anyhow::Result<()> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        let prefix = ObjectPath::from(format!("jobs/{job_id}"));
        let mut listing = store.list(Some(&prefix));
        while let Some(result) = listing.next().await {
            let meta = result?;
            store.delete(&meta.location).await?;
        }
        Ok(())
    }

    pub async fn sync_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };

        let local_files = list_relative_files(repo_root).await?;
        let local_keys = local_files
            .iter()
            .map(|relative_path| self.repo_key(relative_path))
            .collect::<Vec<_>>();

        for relative_path in &local_files {
            let local_path = repo_root.join(relative_path);
            self.store_local_file(self.repo_key(relative_path), &local_path)
                .await?;
        }

        let prefix = ObjectPath::from("repo");
        let mut listing = store.list(Some(&prefix));
        while let Some(result) = listing.next().await {
            let meta = result?;
            if !local_keys.iter().any(|key| key == &meta.location) {
                store.delete(&meta.location).await?;
            }
        }

        Ok(())
    }

    pub async fn restore_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        let prefix = ObjectPath::from("repo");
        let mut listing = store.list(Some(&prefix));
        while let Some(result) = listing.next().await {
            let meta = result?;
            let Some(relative_path) = meta.location.as_ref().strip_prefix("repo/") else {
                continue;
            };
            let local_path = repo_root.join(relative_path);
            self.restore_to_local_path(meta.location, &local_path)
                .await?;
        }
        Ok(())
    }

    pub async fn restore_repo_file(
        &self,
        relative_path: &Path,
        local_path: &Path,
    ) -> anyhow::Result<bool> {
        self.restore_to_local_path(self.repo_key(relative_path), local_path)
            .await
    }

    async fn store_local_file(&self, key: ObjectPath, local_path: &Path) -> anyhow::Result<()> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        let bytes = tokio::fs::read(local_path)
            .await
            .with_context(|| format!("failed to read {}", local_path.display()))?;
        store
            .put(&key, PutPayload::from_bytes(bytes.into()))
            .await
            .with_context(|| format!("failed to upload object {}", key))?;
        debug!(object_key = %key, local_path = %local_path.display(), "uploaded object to storage");
        Ok(())
    }

    async fn restore_to_local_path(
        &self,
        key: ObjectPath,
        local_path: &Path,
    ) -> anyhow::Result<bool> {
        let Some(store) = self.store.as_ref() else {
            return Ok(false);
        };
        let response = match store.get(&key).await {
            Ok(response) => response,
            Err(object_store::Error::NotFound { .. }) => return Ok(false),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to fetch object {}", key));
            }
        };
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("failed to read object {}", key))?;
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(local_path, bytes)
            .await
            .with_context(|| format!("failed to materialize {}", local_path.display()))?;
        Ok(true)
    }

    fn artifact_key(&self, job_id: Uuid, storage_path: &str) -> ObjectPath {
        self.job_key(job_id, "artifacts", storage_path)
    }

    fn log_key(&self, job_id: Uuid, relative_path: &str) -> ObjectPath {
        self.job_key(job_id, "logs", relative_path)
    }

    fn repo_key(&self, relative_path: &Path) -> ObjectPath {
        let sanitized = sanitize_object_path(relative_path.to_string_lossy().as_ref());
        ObjectPath::from(format!("repo/{sanitized}"))
    }

    fn job_key(&self, job_id: Uuid, scope: &str, relative_path: &str) -> ObjectPath {
        let sanitized = sanitize_object_path(relative_path);
        ObjectPath::from(format!("jobs/{job_id}/{scope}/{sanitized}"))
    }
}

#[async_trait]
impl WorkerOutputStorage for JobObjectStorage {
    async fn store_job_artifact(
        &self,
        job_id: Uuid,
        storage_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        JobObjectStorage::store_job_artifact(self, job_id, storage_path, local_path).await
    }

    async fn store_job_log(
        &self,
        job_id: Uuid,
        relative_path: &str,
        local_path: &Path,
    ) -> anyhow::Result<()> {
        JobObjectStorage::store_job_log(self, job_id, relative_path, local_path).await
    }
}

#[async_trait]
impl RepoFileStorage for RuntimeRepoAdapter {
    async fn restore_repo_file(
        &self,
        relative_repo_path: &Path,
        destination_path: &Path,
    ) -> anyhow::Result<bool> {
        self.object_storage
            .restore_repo_file(relative_repo_path, destination_path)
            .await
    }
}

impl RuntimeRepoAdapter {
    pub async fn sync_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        self.object_storage.sync_repo_tree(repo_root).await
    }
}

fn build_s3_store(config: &ObjectStorageConfig) -> anyhow::Result<object_store::aws::AmazonS3> {
    let mut builder = AmazonS3Builder::new()
        .with_endpoint(config.endpoint.as_str())
        .with_region(config.region.as_str())
        .with_bucket_name(config.bucket.as_str())
        .with_access_key_id(config.access_key_id.as_str())
        .with_secret_access_key(config.secret_access_key.as_str())
        .with_virtual_hosted_style_request(!config.path_style);
    if config.endpoint.starts_with("http://") {
        builder = builder.with_allow_http(true);
    }
    builder
        .build()
        .map_err(|error| anyhow::anyhow!("failed to initialize object storage client: {error}"))
}

fn sanitize_object_path(path: &str) -> String {
    let parts = Path::new(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "root".to_string()
    } else {
        parts.join("/")
    }
}

async fn list_relative_files(root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if !tokio::fs::try_exists(root).await? {
        return Ok(files);
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                let relative_path = path
                    .strip_prefix(root)
                    .with_context(|| format!("failed to strip repo root {}", root.display()))?
                    .to_path_buf();
                files.push(relative_path);
            }
        }
    }
    files.sort();
    Ok(files)
}
