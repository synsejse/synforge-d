use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use synforge_core::{
    model::{BuildArtifact, WorkerJobPayload, WorkerResult},
    protocol::{WorkerWireMessage, decode_worker_wire_message, encode_worker_wire_message},
};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Clone)]
pub struct WorkerTransportHandle {
    framed: Arc<Mutex<Framed<TcpStream, LengthDelimitedCodec>>>,
    socket_timeout: Duration,
}

impl WorkerTransportHandle {
    pub(crate) async fn connect(
        connect_addr: &str,
        worker_id: &str,
        socket_timeout: Duration,
    ) -> anyhow::Result<Self> {
        let stream = tokio::time::timeout(socket_timeout, TcpStream::connect(connect_addr))
            .await
            .with_context(|| format!("timed out connecting worker socket {}", connect_addr))?
            .with_context(|| format!("failed to connect worker socket {}", connect_addr))?;
        let framed = Arc::new(Mutex::new(Framed::new(stream, LengthDelimitedCodec::new())));
        let transport = Self {
            framed,
            socket_timeout,
        };
        transport
            .send_message(WorkerWireMessage::Hello {
                worker_id: worker_id.to_string(),
            })
            .await?;
        Ok(transport)
    }

    pub(crate) async fn receive_assignment(&self) -> anyhow::Result<WorkerJobPayload> {
        match self.read_message().await? {
            WorkerWireMessage::JobAssignment { payload } => Ok(*payload),
            WorkerWireMessage::Error { message } => Err(anyhow::anyhow!(message)),
            other => Err(anyhow::anyhow!(
                "unexpected worker assignment message: {:?}",
                other
            )),
        }
    }

    pub(crate) async fn send_log_chunk(&self, path: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.send_message(WorkerWireMessage::LogChunk {
            path: path.to_string(),
            bytes,
        })
        .await
    }

    pub(crate) async fn send_artifact(
        &self,
        artifact_root: &Path,
        artifact: &BuildArtifact,
    ) -> anyhow::Result<()> {
        let path = artifact.file.to_string_lossy().to_string();
        let storage_path = artifact.storage_path().to_string_lossy().to_string();
        self.send_message(WorkerWireMessage::ArtifactStart {
            artifact_id: artifact.id,
            path,
            storage_path: storage_path.clone(),
            kind: artifact.kind,
        })
        .await?;
        let bytes = tokio::fs::read(artifact_root.join(storage_path)).await?;
        for chunk in bytes.chunks(64 * 1024) {
            self.send_message(WorkerWireMessage::ArtifactChunk {
                bytes: chunk.to_vec(),
            })
            .await?;
        }
        self.send_message(WorkerWireMessage::ArtifactComplete).await
    }

    pub(crate) async fn send_result(&self, result: WorkerResult) -> anyhow::Result<()> {
        self.send_message(WorkerWireMessage::Result { result })
            .await
    }

    pub(crate) async fn send_heartbeat(&self) -> anyhow::Result<()> {
        self.send_message(WorkerWireMessage::Heartbeat).await
    }

    async fn send_message(&self, message: WorkerWireMessage) -> anyhow::Result<()> {
        let bytes = encode_worker_wire_message(&message)?;
        let mut framed = self.framed.lock().await;
        framed.send(Bytes::from(bytes)).await?;
        Ok(())
    }

    async fn read_message(&self) -> anyhow::Result<WorkerWireMessage> {
        let mut framed = self.framed.lock().await;
        let bytes = tokio::time::timeout(self.socket_timeout, framed.next())
            .await
            .context("timed out waiting for worker socket message")?
            .ok_or_else(|| anyhow::anyhow!("worker socket disconnected"))??;
        decode_worker_wire_message(bytes.as_ref())
    }
}
