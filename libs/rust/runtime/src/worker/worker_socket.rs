use anyhow::Context;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use synforge_core::{
    model::ArtifactKind,
    protocol::{WorkerWireMessage, decode_worker_wire_message, encode_worker_wire_message},
};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::task::TaskTracker;
use tracing::{error, warn};

use crate::worker::sessions::WorkerSessionBroker;
use synforge_store::{DieselStore, JobStore};

pub fn start_worker_listener(
    listen_addr: String,
    store: DieselStore,
    sessions: WorkerSessionBroker,
    task_tracker: TaskTracker,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    task_tracker.clone().spawn(async move {
        let listener = match TcpListener::bind(&listen_addr).await {
            Ok(listener) => listener,
            Err(error) => {
                error!("failed to bind worker listener {}: {}", listen_addr, error);
                return;
            }
        };
        tracing::info!("worker socket listening on {}", listen_addr);
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    break;
                }
                accepted = listener.accept() => match accepted {
                Ok((stream, peer)) => {
                    let store = store.clone();
                    let sessions = sessions.clone();
                    let task_tracker = task_tracker.clone();
                    task_tracker.spawn(async move {
                        if let Err(error) = handle_connection(stream, store, sessions).await {
                            warn!("worker socket {} failed: {}", peer, error);
                        }
                    });
                }
                Err(error) => warn!("worker socket accept failed: {}", error),
                }
            }
        }
    });
}

async fn handle_connection(
    stream: TcpStream,
    store: DieselStore,
    sessions: WorkerSessionBroker,
) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    let hello = read_message(&mut framed).await?;
    let WorkerWireMessage::Hello { worker_id } = hello else {
        anyhow::bail!("expected worker hello");
    };

    let (job_id, payload) = sessions.connect_worker(&worker_id).await?;

    write_message(
        &mut framed,
        &WorkerWireMessage::JobAssignment {
            payload: Box::new(payload),
        },
    )
    .await?;

    let mut current_artifact: Option<ActiveArtifactUpload> = None;
    let mut log_files: HashMap<String, tokio::fs::File> = HashMap::new();
    while let Some(frame) = framed.next().await {
        let frame = frame?;
        let message: WorkerWireMessage = decode_worker_wire_message(frame.as_ref())?;
        match message {
            WorkerWireMessage::LogChunk { path, bytes } => {
                let upload_path = sessions.log_storage_path(job_id, &path);
                if let Some(parent) = upload_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let created = if log_files.contains_key(&path) {
                    false
                } else {
                    let file = tokio::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&upload_path)
                        .await
                        .with_context(|| format!("failed to open {}", upload_path.display()))?;
                    log_files.insert(path.clone(), file);
                    true
                };
                if created {
                    store
                        .upsert_build_log(job_id, &path)
                        .await
                        .with_context(|| {
                            format!("failed to register log source {} for job {}", path, job_id)
                        })?;
                }
                let file = log_files
                    .get_mut(&path)
                    .ok_or_else(|| anyhow::anyhow!("log file handle disappeared for {}", path))?;
                file.write_all(&bytes).await?;
                file.flush().await?;
            }
            WorkerWireMessage::ArtifactStart {
                artifact_id,
                path,
                storage_path,
                kind,
            } => {
                if current_artifact.is_some() {
                    anyhow::bail!("artifact upload already in progress");
                }
                let upload_path = sessions.artifact_storage_path(job_id, &storage_path);
                if let Some(parent) = upload_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let handle = tokio::fs::File::create(&upload_path).await?;
                current_artifact = Some(ActiveArtifactUpload {
                    artifact_id,
                    file: path,
                    storage_path,
                    kind,
                    handle,
                });
            }
            WorkerWireMessage::ArtifactChunk { bytes } => {
                let upload = current_artifact
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("artifact chunk received without start"))?;
                upload.handle.write_all(&bytes).await?;
            }
            WorkerWireMessage::ArtifactComplete => {
                let Some(mut upload) = current_artifact.take() else {
                    anyhow::bail!("artifact complete received without start");
                };
                upload.handle.flush().await?;
                sessions
                    .finalize_artifact_upload(
                        job_id,
                        upload.artifact_id,
                        &upload.file,
                        &upload.storage_path,
                        upload.kind,
                    )
                    .await?;
            }
            WorkerWireMessage::Result { result } => {
                sessions.complete(job_id, result).await?;
                return Ok(());
            }
            WorkerWireMessage::Heartbeat => {}
            WorkerWireMessage::Error { message } => {
                anyhow::bail!("worker reported error: {}", message);
            }
            WorkerWireMessage::Hello { .. } | WorkerWireMessage::JobAssignment { .. } => {
                anyhow::bail!("unexpected worker message");
            }
        }
    }

    Ok(())
}

async fn read_message(
    framed: &mut Framed<TcpStream, LengthDelimitedCodec>,
) -> anyhow::Result<WorkerWireMessage> {
    let bytes = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("worker disconnected before hello"))??;
    decode_worker_wire_message(bytes.as_ref())
}

async fn write_message(
    framed: &mut Framed<TcpStream, LengthDelimitedCodec>,
    message: &WorkerWireMessage,
) -> anyhow::Result<()> {
    let bytes = encode_worker_wire_message(message)?;
    framed.send(Bytes::from(bytes)).await?;
    Ok(())
}

struct ActiveArtifactUpload {
    artifact_id: uuid::Uuid,
    file: String,
    storage_path: String,
    kind: ArtifactKind,
    handle: tokio::fs::File,
}
