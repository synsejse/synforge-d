use anyhow::Context;
use rkyv::rancor::Error as RkyvError;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{WorkerJobPayload, WorkerResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerWireMessage {
    Hello { worker_id: String },
    JobAssignment { payload: Box<WorkerJobPayload> },
    Heartbeat,
    LogChunk { path: String, bytes: Vec<u8> },
    ArtifactStart { artifact_id: Uuid, file: String },
    ArtifactChunk { bytes: Vec<u8> },
    ArtifactComplete,
    Result { result: WorkerResult },
    ResultAck,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactStartPayload {
    artifact_id: Uuid,
    file: String,
}

#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize, PartialEq, Eq)]
enum WorkerWireEnvelope {
    Hello { worker_id: String },
    JobAssignment { payload_json: Vec<u8> },
    Heartbeat,
    LogChunk { path: String, bytes: Vec<u8> },
    ArtifactStart { payload_json: Vec<u8> },
    ArtifactChunk { bytes: Vec<u8> },
    ArtifactComplete,
    Result { result_json: Vec<u8> },
    ResultAck,
    Error { message: String },
}

impl TryFrom<&WorkerWireMessage> for WorkerWireEnvelope {
    type Error = anyhow::Error;

    fn try_from(message: &WorkerWireMessage) -> Result<Self, anyhow::Error> {
        Ok(match message {
            WorkerWireMessage::Hello { worker_id } => Self::Hello {
                worker_id: worker_id.clone(),
            },
            WorkerWireMessage::JobAssignment { payload } => Self::JobAssignment {
                payload_json: serde_json::to_vec(payload.as_ref())
                    .context("failed to serialize worker assignment payload")?,
            },
            WorkerWireMessage::Heartbeat => Self::Heartbeat,
            WorkerWireMessage::LogChunk { path, bytes } => Self::LogChunk {
                path: path.clone(),
                bytes: bytes.clone(),
            },
            WorkerWireMessage::ArtifactStart { artifact_id, file } => Self::ArtifactStart {
                payload_json: serde_json::to_vec(&ArtifactStartPayload {
                    artifact_id: *artifact_id,
                    file: file.clone(),
                })
                .context("failed to serialize artifact start payload")?,
            },
            WorkerWireMessage::ArtifactChunk { bytes } => Self::ArtifactChunk {
                bytes: bytes.clone(),
            },
            WorkerWireMessage::ArtifactComplete => Self::ArtifactComplete,
            WorkerWireMessage::Result { result } => Self::Result {
                result_json: serde_json::to_vec(result)
                    .context("failed to serialize worker result payload")?,
            },
            WorkerWireMessage::ResultAck => Self::ResultAck,
            WorkerWireMessage::Error { message } => Self::Error {
                message: message.clone(),
            },
        })
    }
}

impl TryFrom<WorkerWireEnvelope> for WorkerWireMessage {
    type Error = anyhow::Error;

    fn try_from(envelope: WorkerWireEnvelope) -> Result<Self, anyhow::Error> {
        Ok(match envelope {
            WorkerWireEnvelope::Hello { worker_id } => Self::Hello { worker_id },
            WorkerWireEnvelope::JobAssignment { payload_json } => Self::JobAssignment {
                payload: Box::new(
                    serde_json::from_slice(&payload_json)
                        .context("failed to decode worker assignment payload")?,
                ),
            },
            WorkerWireEnvelope::Heartbeat => Self::Heartbeat,
            WorkerWireEnvelope::LogChunk { path, bytes } => Self::LogChunk { path, bytes },
            WorkerWireEnvelope::ArtifactStart { payload_json } => {
                let payload: ArtifactStartPayload = serde_json::from_slice(&payload_json)
                    .context("failed to decode artifact start payload")?;
                Self::ArtifactStart {
                    artifact_id: payload.artifact_id,
                    file: payload.file,
                }
            }
            WorkerWireEnvelope::ArtifactChunk { bytes } => Self::ArtifactChunk { bytes },
            WorkerWireEnvelope::ArtifactComplete => Self::ArtifactComplete,
            WorkerWireEnvelope::Result { result_json } => Self::Result {
                result: serde_json::from_slice(&result_json)
                    .context("failed to decode worker result payload")?,
            },
            WorkerWireEnvelope::ResultAck => Self::ResultAck,
            WorkerWireEnvelope::Error { message } => Self::Error { message },
        })
    }
}

pub fn encode_worker_wire_message(message: &WorkerWireMessage) -> anyhow::Result<Vec<u8>> {
    let envelope = WorkerWireEnvelope::try_from(message)?;
    let bytes = rkyv::to_bytes::<RkyvError>(&envelope)
        .context("failed to serialize worker wire envelope with rkyv")?;
    Ok(bytes.to_vec())
}

pub fn decode_worker_wire_message(bytes: &[u8]) -> anyhow::Result<WorkerWireMessage> {
    let envelope = rkyv::from_bytes::<WorkerWireEnvelope, RkyvError>(bytes)
        .context("failed to deserialize worker wire envelope with rkyv")?;
    WorkerWireMessage::try_from(envelope)
}

#[cfg(test)]
#[path = "worker_protocol_tests.rs"]
mod tests;
