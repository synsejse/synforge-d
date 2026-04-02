use anyhow::Context;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, warn};

use crate::sessions::WorkerSessionBroker;

pub fn start_worker_listener(listen_addr: String, sessions: WorkerSessionBroker) {
    tokio::spawn(async move {
        let listener = match TcpListener::bind(&listen_addr).await {
            Ok(listener) => listener,
            Err(error) => {
                error!("failed to bind worker listener {}: {}", listen_addr, error);
                return;
            }
        };
        tracing::info!("worker socket listening on {}", listen_addr);
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let sessions = sessions.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_connection(stream, sessions).await {
                            warn!("worker socket {} failed: {}", peer, error);
                        }
                    });
                }
                Err(error) => warn!("worker socket accept failed: {}", error),
            }
        }
    });
}

async fn handle_connection(stream: TcpStream, sessions: WorkerSessionBroker) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    let hello = read_message(&mut framed).await?;
    let synforge_core::WorkerWireMessage::Hello { worker_id } = hello else {
        anyhow::bail!("expected worker hello");
    };

    let (job_id, payload) = sessions.connect_worker(&worker_id).await?;
    let log_path = sessions.begin_log_stream(job_id).await?;
    let mut log_file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
        .with_context(|| format!("failed to open {}", log_path.display()))?;

    write_message(
        &mut framed,
        &synforge_core::WorkerWireMessage::JobAssignment { payload },
    )
    .await?;

    let mut current_artifact: Option<ActiveArtifactUpload> = None;
    while let Some(frame) = framed.next().await {
        let frame = frame?;
        let message: synforge_core::WorkerWireMessage = bincode::deserialize(&frame)?;
        match message {
            synforge_core::WorkerWireMessage::LogChunk { bytes } => {
                log_file.write_all(&bytes).await?;
                log_file.flush().await?;
            }
            synforge_core::WorkerWireMessage::ArtifactStart { path, kind } => {
                if current_artifact.is_some() {
                    anyhow::bail!("artifact upload already in progress");
                }
                let upload_path = sessions.artifact_upload_path(job_id, &path);
                if let Some(parent) = upload_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let file = tokio::fs::File::create(&upload_path).await?;
                current_artifact = Some(ActiveArtifactUpload {
                    relative_path: path,
                    kind,
                    file,
                });
            }
            synforge_core::WorkerWireMessage::ArtifactChunk { bytes } => {
                let upload = current_artifact
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("artifact chunk received without start"))?;
                upload.file.write_all(&bytes).await?;
            }
            synforge_core::WorkerWireMessage::ArtifactComplete => {
                let Some(mut upload) = current_artifact.take() else {
                    anyhow::bail!("artifact complete received without start");
                };
                upload.file.flush().await?;
                sessions
                    .finalize_artifact_upload(job_id, &upload.relative_path, upload.kind)
                    .await?;
            }
            synforge_core::WorkerWireMessage::Result { result } => {
                sessions.complete(job_id, result).await?;
                return Ok(());
            }
            synforge_core::WorkerWireMessage::Error { message } => {
                anyhow::bail!("worker reported error: {}", message);
            }
            synforge_core::WorkerWireMessage::Hello { .. }
            | synforge_core::WorkerWireMessage::JobAssignment { .. } => {
                anyhow::bail!("unexpected worker message");
            }
        }
    }

    Ok(())
}

async fn read_message(
    framed: &mut Framed<TcpStream, LengthDelimitedCodec>,
) -> anyhow::Result<synforge_core::WorkerWireMessage> {
    let bytes = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("worker disconnected before hello"))??;
    Ok(bincode::deserialize(&bytes)?)
}

async fn write_message(
    framed: &mut Framed<TcpStream, LengthDelimitedCodec>,
    message: &synforge_core::WorkerWireMessage,
) -> anyhow::Result<()> {
    let bytes = bincode::serialize(message)?;
    framed.send(Bytes::from(bytes)).await?;
    Ok(())
}

struct ActiveArtifactUpload {
    relative_path: String,
    kind: synforge_core::ArtifactKind,
    file: tokio::fs::File,
}
