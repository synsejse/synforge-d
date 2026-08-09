use uuid::Uuid;

use super::{WorkerWireMessage, decode_worker_wire_message, encode_worker_wire_message};

#[test]
fn artifact_start_round_trip_contains_only_artifact_identity() {
    let message = WorkerWireMessage::ArtifactStart {
        artifact_id: Uuid::now_v7(),
        file: "package.rpm".to_string(),
    };

    let bytes = encode_worker_wire_message(&message).expect("encode artifact start");
    let decoded = decode_worker_wire_message(&bytes).expect("decode artifact start");
    assert_eq!(decoded, message);
}
