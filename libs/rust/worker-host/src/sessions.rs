use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use synforge_publish::{WorkerOutputStorage, WorkerOutputUpload};
use tokio::sync::{Mutex, Notify};
use tracing::warn;
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
    output_storage: Arc<dyn WorkerOutputStorage>,
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
    pub fn new(root: PathBuf, output_storage: Arc<dyn WorkerOutputStorage>) -> Self {
        Self {
            root,
            output_storage,
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

    pub async fn begin_remote_artifact_upload(
        &self,
        job_id: Uuid,
        storage_path: &str,
    ) -> anyhow::Result<Box<dyn WorkerOutputUpload>> {
        self.output_storage
            .begin_job_artifact_upload(job_id, storage_path)
            .await
    }

    pub async fn finalize_artifact_upload(
        &self,
        job_id: Uuid,
        artifact_id: Uuid,
        file: &str,
        kind: ArtifactKind,
        sha256: String,
        size_bytes: u64,
    ) -> anyhow::Result<BuildArtifact> {
        let entry = self
            .state
            .get(&job_id)
            .ok_or_else(|| anyhow::anyhow!("worker session {} not found", job_id))?;
        let (package_name, mock_chroot) = build_metadata_from_payload(&entry.payload)?;
        let artifact = BuildArtifact {
            id: artifact_id,
            package_name,
            mock_chroot,
            file: PathBuf::from(file),
            sha256,
            size_bytes,
            kind,
            signing_status: None,
            signing_error_message: None,
        };
        entry.artifacts.lock().await.push(artifact.clone());
        Ok(artifact)
    }

    pub fn log_storage_path(&self, job_id: Uuid, relative_path: &str) -> PathBuf {
        let relative = Path::new(relative_path);
        let sanitized = relative
            .components()
            .filter_map(|component| match component {
                std::path::Component::Normal(part) => Some(PathBuf::from(part)),
                _ => None,
            })
            .fold(PathBuf::new(), |mut acc, part| {
                acc.push(part);
                acc
            });
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
        let log_sources = entry.log_sources.lock().await.clone();
        self.archive_session_outputs_in_background(job_id, &log_sources);
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

    fn archive_session_outputs_in_background(&self, job_id: Uuid, log_sources: &[String]) {
        let output_storage = Arc::clone(&self.output_storage);
        let log_uploads = log_sources
            .iter()
            .map(|log_source| {
                let local_path = self.log_storage_path(job_id, log_source);
                (log_source.clone(), local_path)
            })
            .collect::<Vec<_>>();

        tokio::spawn(async move {
            for (log_source, local_path) in log_uploads {
                if let Err(error) = output_storage
                    .store_job_log(job_id, &log_source, &local_path)
                    .await
                {
                    warn!(
                        job_id = %job_id,
                        log_source,
                        error = %error,
                        "failed to archive job log to object storage"
                    );
                }
            }
        });
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
