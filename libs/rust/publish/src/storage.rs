use std::collections::HashSet;
use std::path::{Component, Path};
use std::sync::Arc;
use std::{fmt, fmt::Formatter};

use anyhow::Context;
use async_trait::async_trait;
use futures_util::StreamExt;
use object_store::{
    ObjectStore, PutPayload, WriteMultipart, aws::AmazonS3Builder, path::Path as ObjectPath,
};
use synforge_core::config::{DaemonConfig, ObjectStorageConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;
use uuid::Uuid;

use crate::{RepoFileStorage, RuntimeRepoAdapter};

#[async_trait]
pub trait WorkerOutputStorage: Send + Sync {
    async fn begin_job_artifact_upload(
        &self,
        job_id: Uuid,
        storage_path: &str,
    ) -> anyhow::Result<Box<dyn WorkerOutputUpload>>;

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

#[async_trait]
pub trait WorkerOutputUpload: Send {
    async fn write_chunk(&mut self, bytes: &[u8]) -> anyhow::Result<()>;

    async fn finish(self: Box<Self>) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct JobObjectStorage {
    store: Arc<dyn ObjectStore>,
}

impl fmt::Debug for JobObjectStorage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobObjectStorage")
            .field("enabled", &true)
            .finish()
    }
}

impl JobObjectStorage {
    pub async fn from_config(config: &DaemonConfig) -> anyhow::Result<Self> {
        let store = build_s3_store(&config.object_storage)?;
        Ok(Self {
            store: Arc::new(store),
        })
    }

    pub async fn begin_job_artifact_upload(
        &self,
        job_id: Uuid,
        storage_path: &str,
    ) -> anyhow::Result<Box<dyn WorkerOutputUpload>> {
        let key = self.artifact_key(job_id, storage_path);
        let upload = self
            .store
            .put_multipart(&key)
            .await
            .with_context(|| format!("failed to start multipart upload for object {}", key))?;
        Ok(Box::new(ObjectMultipartUpload {
            store: Arc::clone(&self.store),
            key,
            writer: Some(WriteMultipart::new(upload)),
            bytes_written: 0,
        }))
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        let mut listing = self.store.list(Some(&ObjectPath::from("jobs")));
        if let Some(result) = listing.next().await {
            result?;
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
        let prefix = ObjectPath::from(format!("jobs/{job_id}"));
        let mut listing = self.store.list(Some(&prefix));
        while let Some(result) = listing.next().await {
            let meta = result?;
            self.store.delete(&meta.location).await?;
        }
        Ok(())
    }

    pub async fn sync_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        let local_files = list_relative_files(repo_root).await?;
        let local_keys = local_files
            .iter()
            .map(|relative_path| self.repo_key(relative_path))
            .collect::<HashSet<_>>();

        for relative_path in &local_files {
            let local_path = repo_root.join(relative_path);
            self.store_local_file(self.repo_key(relative_path), &local_path)
                .await?;
        }

        let prefix = ObjectPath::from("repo");
        let mut listing = self.store.list(Some(&prefix));
        while let Some(result) = listing.next().await {
            let meta = result?;
            if !local_keys.contains(&meta.location) {
                self.store.delete(&meta.location).await?;
            }
        }

        Ok(())
    }

    pub async fn restore_repo_tree(&self, repo_root: &Path) -> anyhow::Result<()> {
        let prefix = ObjectPath::from("repo");
        let mut listing = self.store.list(Some(&prefix));
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
        let metadata = tokio::fs::metadata(local_path)
            .await
            .with_context(|| format!("failed to stat {}", local_path.display()))?;
        if metadata.len() == 0 {
            self.store
                .put(&key, PutPayload::default())
                .await
                .with_context(|| format!("failed to upload object {}", key))?;
            return Ok(());
        }

        let mut file = tokio::fs::File::open(local_path)
            .await
            .with_context(|| format!("failed to read {}", local_path.display()))?;
        let upload = self
            .store
            .put_multipart(&key)
            .await
            .with_context(|| format!("failed to start multipart upload for object {}", key))?;
        let mut writer = WriteMultipart::new(upload);
        let mut buffer = vec![0_u8; 1024 * 1024];
        loop {
            writer.wait_for_capacity(2).await?;
            let read = file
                .read(&mut buffer)
                .await
                .with_context(|| format!("failed to read {}", local_path.display()))?;
            if read == 0 {
                break;
            }
            writer.write(&buffer[..read]);
        }
        writer
            .finish()
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
        let response = match self.store.get(&key).await {
            Ok(response) => response,
            Err(object_store::Error::NotFound { .. }) => return Ok(false),
            Err(error) => {
                return Err(error).with_context(|| format!("failed to fetch object {}", key));
            }
        };
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut output = tokio::fs::File::create(local_path)
            .await
            .with_context(|| format!("failed to materialize {}", local_path.display()))?;
        let mut stream = response.into_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.with_context(|| format!("failed to read object {}", key))?;
            output
                .write_all(&bytes)
                .await
                .with_context(|| format!("failed to materialize {}", local_path.display()))?;
        }
        output
            .flush()
            .await
            .with_context(|| format!("failed to flush {}", local_path.display()))?;
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
    async fn begin_job_artifact_upload(
        &self,
        job_id: Uuid,
        storage_path: &str,
    ) -> anyhow::Result<Box<dyn WorkerOutputUpload>> {
        JobObjectStorage::begin_job_artifact_upload(self, job_id, storage_path).await
    }

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

struct ObjectMultipartUpload {
    store: Arc<dyn ObjectStore>,
    key: ObjectPath,
    writer: Option<WriteMultipart>,
    bytes_written: u64,
}

#[async_trait]
impl WorkerOutputUpload for ObjectMultipartUpload {
    async fn write_chunk(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.bytes_written += bytes.len() as u64;
        if bytes.is_empty() {
            return Ok(());
        }
        let Some(writer) = self.writer.as_mut() else {
            anyhow::bail!("object upload {} already finished", self.key);
        };
        writer.wait_for_capacity(2).await?;
        writer.write(bytes);
        Ok(())
    }

    async fn finish(mut self: Box<Self>) -> anyhow::Result<()> {
        let Some(writer) = self.writer.take() else {
            return Ok(());
        };
        if self.bytes_written == 0 {
            writer
                .abort()
                .await
                .with_context(|| format!("failed to abort empty multipart upload {}", self.key))?;
            self.store
                .put(&self.key, PutPayload::default())
                .await
                .with_context(|| format!("failed to upload empty object {}", self.key))?;
            debug!(object_key = %self.key, "streamed empty artifact upload to object storage");
            return Ok(());
        }
        writer
            .finish()
            .await
            .with_context(|| format!("failed to upload object {}", self.key))?;
        debug!(object_key = %self.key, "streamed artifact upload to object storage");
        Ok(())
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
