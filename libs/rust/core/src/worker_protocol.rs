use serde::{Deserialize, Serialize};

use crate::model::{ArtifactKind, WorkerJobPayload, WorkerResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerWireMessage {
    Hello { worker_id: String },
    JobAssignment { payload: WorkerJobPayload },
    Heartbeat,
    LogChunk { path: String, bytes: Vec<u8> },
    ArtifactStart { path: String, kind: ArtifactKind },
    ArtifactChunk { bytes: Vec<u8> },
    ArtifactComplete,
    Result { result: WorkerResult },
    Error { message: String },
}
