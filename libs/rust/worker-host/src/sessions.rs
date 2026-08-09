use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use path_clean::PathClean;
use sha2::Digest;
use strict_path::{PathBoundary, StrictPath};
use tokio::io::AsyncReadExt;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

use synforge_core::{
    model::{
        ArtifactKind, BuildArtifact, BuildCcacheStats, BuildStatus, WorkerAction,
        WorkerBuildResult, WorkerJobPayload, WorkerResult,
    },
    package::{is_safe_path_segment, parse_mock_chroot},
    validated::PackageName,
};

#[derive(Clone)]
pub struct WorkerSessionBroker {
    root: PathBuf,
    state: Arc<DashMap<Uuid, Arc<WorkerSessionEntry>>>,
}

struct WorkerSessionEntry {
    worker_id: String,
    payload: WorkerJobPayload,
    container_id: Mutex<Option<String>>,
    artifacts: Mutex<Vec<BuildArtifact>>,
    log_sources: Mutex<Vec<String>>,
    result: Mutex<Option<WorkerResult>>,
    notify: Arc<Notify>,
}

#[derive(Debug, Clone)]
pub struct WorkerSession {
    pub worker_id: String,
}

#[derive(Debug, Clone)]
pub struct ActiveWorkerSession {
    pub job_id: Uuid,
    pub container_id: String,
}

impl WorkerSessionBroker {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            state: Arc::new(DashMap::new()),
        }
    }

    pub async fn create_session(
        &self,
        job_id: Uuid,
        payload: WorkerJobPayload,
    ) -> anyhow::Result<WorkerSession> {
        let worker_id = job_id.to_string();
        let job_root = self.job_root(job_id);
        tokio::fs::create_dir_all(job_root.join("artifacts")).await?;
        tokio::fs::create_dir_all(job_root.join("logs")).await?;
        self.state.insert(
            job_id,
            Arc::new(WorkerSessionEntry {
                worker_id: worker_id.clone(),
                payload,
                container_id: Mutex::new(None),
                artifacts: Mutex::new(Vec::new()),
                log_sources: Mutex::new(Vec::new()),
                result: Mutex::new(None),
                notify: Arc::new(Notify::new()),
            }),
        );
        Ok(WorkerSession { worker_id })
    }

    pub async fn set_container_id(&self, job_id: Uuid, container_id: String) {
        if let Some(entry) = self.state.get(&job_id) {
            *entry.container_id.lock().await = Some(container_id);
        }
    }

    pub async fn container_id_for_job(&self, job_id: Uuid) -> Option<String> {
        let entry = self.state.get(&job_id)?;
        entry.container_id.lock().await.clone()
    }

    pub async fn connect_worker(
        &self,
        worker_id: &str,
    ) -> anyhow::Result<(Uuid, WorkerJobPayload)> {
        for entry in self.state.iter() {
            if entry.worker_id == worker_id {
                return Ok((*entry.key(), entry.payload.clone()));
            }
        }
        Err(anyhow::anyhow!("worker session {} not found", worker_id))
    }

    pub async fn active_container_sessions(&self) -> Vec<ActiveWorkerSession> {
        let mut sessions = Vec::new();
        for entry in self.state.iter() {
            let container_id = entry.container_id.lock().await;
            if let Some(id) = container_id.as_ref() {
                sessions.push(ActiveWorkerSession {
                    job_id: *entry.key(),
                    container_id: id.clone(),
                });
            }
        }
        sessions
    }

    pub fn active_job_ids(&self) -> Vec<Uuid> {
        self.state.iter().map(|entry| *entry.key()).collect()
    }

    pub fn artifact_storage_path(
        &self,
        job_id: Uuid,
        file: &str,
    ) -> anyhow::Result<(PathBuf, StrictPath)> {
        let entry = self
            .state
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("worker session {} not found", job_id))?;
        let (_, mock_chroot) = build_metadata_from_payload(&entry.payload)?;
        if !is_safe_path_segment(file) {
            anyhow::bail!("artifact filename {file:?} must be a single safe path segment");
        }

        let file = PathBuf::from(file);
        let relative_path = PathBuf::from(mock_chroot).join(&file);
        let artifact_root = self.job_root(job_id).join("artifacts");
        let boundary = PathBoundary::try_new(&artifact_root)?;
        let storage_path = boundary.strict_join(relative_path)?;
        Ok((file, storage_path))
    }

    pub async fn finalize_artifact_upload(
        &self,
        job_id: Uuid,
        artifact_id: Uuid,
        file: PathBuf,
        storage_path: &StrictPath,
    ) -> anyhow::Result<BuildArtifact> {
        let entry = self
            .state
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("worker session {} not found", job_id))?;
        let (package_name, mock_chroot) = build_metadata_from_payload(&entry.payload)?;
        let mut stored_file = tokio::fs::File::open(storage_path.interop_path()).await?;
        let mut hasher = sha2::Sha256::new();
        let mut size_bytes = 0_u64;
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = stored_file.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            size_bytes += read as u64;
        }
        let artifact = BuildArtifact {
            id: artifact_id,
            package_name,
            mock_chroot,
            kind: file
                .to_str()
                .map(ArtifactKind::from_file_name)
                .unwrap_or(ArtifactKind::Other),
            file,
            sha256: hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            size_bytes,
            signing_status: None,
            signing_error_message: None,
        };
        let mut artifacts = entry.artifacts.lock().await;
        if artifacts
            .iter()
            .any(|existing| existing.id == artifact.id || existing.file == artifact.file)
        {
            anyhow::bail!(
                "duplicate artifact upload for id {} or file {}",
                artifact.id,
                artifact.file.display()
            );
        }
        artifacts.push(artifact.clone());
        Ok(artifact)
    }

    pub fn log_storage_path(&self, job_id: Uuid, relative_path: &str) -> PathBuf {
        let sanitized = sanitize_relative_path(relative_path);
        self.job_root(job_id).join("logs").join(sanitized)
    }

    pub async fn register_log_source(&self, job_id: Uuid, relative_path: &str) {
        let Some(entry) = self.state.get(&job_id) else {
            return;
        };
        let mut sources = entry.log_sources.lock().await;
        if !sources.iter().any(|path| path == relative_path) {
            sources.push(relative_path.to_string());
        }
    }

    pub async fn complete(&self, job_id: Uuid, result: WorkerResult) -> anyhow::Result<()> {
        let entry = self
            .state
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("worker session {} not found", job_id))?;
        let artifacts = entry.artifacts.lock().await;
        let result = merge_result(job_id, &entry.payload, result, &artifacts)?;
        drop(artifacts);
        *entry.result.lock().await = Some(result);
        entry.notify.notify_waiters();
        Ok(())
    }

    pub async fn fail_build_result(&self, job_id: Uuid, message: &str) -> anyhow::Result<bool> {
        let package_name = {
            let Some(entry) = self.state.get(&job_id) else {
                return Ok(false);
            };
            let WorkerAction::Build(build) = &entry.payload.action else {
                return Ok(false);
            };
            build.package_name.clone()
        };
        self.complete(
            job_id,
            WorkerResult::Build(WorkerBuildResult {
                job_id,
                package_name,
                status: BuildStatus::Failed,
                artifacts: Vec::new(),
                message: Some(message.to_string()),
                ccache_stats: None,
            }),
        )
        .await?;
        Ok(true)
    }

    pub async fn wait_for_result(
        &self,
        job_id: Uuid,
        timeout: Duration,
    ) -> anyhow::Result<Option<WorkerResult>> {
        let entry = {
            let Some(entry) = self.state.get(&job_id) else {
                return Ok(None);
            };
            Arc::clone(entry.value())
        };

        tokio::time::timeout(timeout, async {
            loop {
                let notified = entry.notify.notified();
                if let Some(result) = entry.result.lock().await.clone() {
                    return result;
                }
                notified.await;
            }
        })
        .await
        .map(Some)
        .or_else(|_| Ok(None))
    }

    pub fn remove_session(&self, job_id: Uuid) {
        self.state.remove(&job_id);
    }

    pub fn job_root(&self, job_id: Uuid) -> PathBuf {
        self.root.join(job_id.to_string())
    }
}

fn build_metadata_from_payload(payload: &WorkerJobPayload) -> anyhow::Result<(String, String)> {
    match &payload.action {
        WorkerAction::Build(build) => {
            PackageName::new(&build.package.name)?;
            parse_mock_chroot(&build.mock_chroot)
                .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", build.mock_chroot))?;
            Ok((build.package.name.clone(), build.mock_chroot.clone()))
        }
        WorkerAction::Parse(_) => Err(anyhow::anyhow!(
            "artifact upload received for non-build worker session"
        )),
    }
}

fn merge_result(
    job_id: Uuid,
    payload: &WorkerJobPayload,
    result: WorkerResult,
    artifacts: &[BuildArtifact],
) -> anyhow::Result<WorkerResult> {
    match (&payload.action, result) {
        (WorkerAction::Parse(_), WorkerResult::Parse(parse)) => Ok(WorkerResult::Parse(parse)),
        (WorkerAction::Build(build), WorkerResult::Build(result)) => {
            if !matches!(
                result.status,
                BuildStatus::Succeeded | BuildStatus::Failed | BuildStatus::TimedOut
            ) {
                anyhow::bail!("worker returned non-terminal build status")
            }
            Ok(WorkerResult::Build(WorkerBuildResult {
                job_id,
                package_name: build.package.name.clone(),
                status: result.status,
                artifacts: artifacts.to_vec(),
                message: result.message,
                ccache_stats: sanitize_ccache_stats(result.ccache_stats),
            }))
        }
        (WorkerAction::Parse(_), WorkerResult::Build(_)) => {
            anyhow::bail!("build result received for parse worker session")
        }
        (WorkerAction::Build(_), WorkerResult::Parse(_)) => {
            anyhow::bail!("parse result received for build worker session")
        }
    }
}

fn sanitize_ccache_stats(stats: Option<BuildCcacheStats>) -> Option<BuildCcacheStats> {
    stats.filter(|stats| {
        let values = [
            stats.direct_hits,
            stats.preprocessed_hits,
            stats.cache_misses,
            stats.uncacheable_calls,
            stats.error_calls,
        ];
        stats.compiler_calls <= i64::MAX as u64
            && values.iter().all(|value| *value <= i64::MAX as u64)
            && values
                .into_iter()
                .try_fold(0_u64, u64::checked_add)
                .is_some_and(|total| total == stats.compiler_calls)
    })
}

fn sanitize_relative_path(path: &str) -> PathBuf {
    std::path::Path::new(path)
        .clean()
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(PathBuf::from(part)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[path = "sessions_tests.rs"]
mod tests;
