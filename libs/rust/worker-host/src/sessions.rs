use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use path_clean::PathClean;
use sha2::Digest;
use tokio::io::AsyncReadExt;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

use synforge_core::{
    model::{
        ArtifactKind, BuildArtifact, BuildStatus, WorkerAction, WorkerBuildResult,
        WorkerJobPayload, WorkerResult,
    },
    package::parse_mock_chroot,
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

    pub fn artifact_storage_path(&self, job_id: Uuid, relative_path: &str) -> PathBuf {
        let sanitized = sanitize_relative_path(relative_path);
        self.job_root(job_id).join("artifacts").join(sanitized)
    }

    pub async fn finalize_artifact_upload(
        &self,
        job_id: Uuid,
        artifact_id: Uuid,
        file: &str,
        storage_path: &str,
        kind: ArtifactKind,
    ) -> anyhow::Result<BuildArtifact> {
        let entry = self
            .state
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("worker session {} not found", job_id))?;
        let (package_name, mock_chroot) = build_metadata_from_payload(&entry.payload)?;
        let stored_path = self.artifact_storage_path(job_id, storage_path);
        let mut stored_file = tokio::fs::File::open(&stored_path).await?;
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
        if let Some(parent) = stored_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let artifact = BuildArtifact {
            id: artifact_id,
            package_name,
            mock_chroot,
            file: PathBuf::from(file),
            sha256: hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            size_bytes,
            kind,
            signing_status: None,
            signing_error_message: None,
        };
        entry.artifacts.lock().await.push(artifact.clone());
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
        *entry.result.lock().await = Some(merge_result(result, &artifacts));
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
            parse_mock_chroot(&build.mock_chroot)
                .ok_or_else(|| anyhow::anyhow!("invalid mock chroot {}", build.mock_chroot))?;
            Ok((build.package.name.clone(), build.mock_chroot.clone()))
        }
        WorkerAction::Parse(_) => Err(anyhow::anyhow!(
            "artifact upload received for non-build worker session"
        )),
    }
}

fn merge_result(result: WorkerResult, artifacts: &[BuildArtifact]) -> WorkerResult {
    match result {
        WorkerResult::Parse(parse) => WorkerResult::Parse(parse),
        WorkerResult::Build(build) => WorkerResult::Build(WorkerBuildResult {
            artifacts: if build.artifacts.is_empty() {
                artifacts.to_vec()
            } else {
                build.artifacts
            },
            ..build
        }),
    }
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
