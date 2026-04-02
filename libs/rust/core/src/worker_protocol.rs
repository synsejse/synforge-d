use serde::{Deserialize, Serialize};

use crate::WorkerJobPayload;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerWireMessage {
    Hello { worker_id: String },
    JobAssignment { payload: WorkerJobPayload },
    LogChunk { bytes: Vec<u8> },
    ArtifactStart { path: String, kind: crate::ArtifactKind },
    ArtifactChunk { bytes: Vec<u8> },
    ArtifactComplete,
    Result { result: crate::WorkerResult },
    Error { message: String },
}
