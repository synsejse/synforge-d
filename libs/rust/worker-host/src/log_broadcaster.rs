use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use uuid::Uuid;

/// Bounded capacity for each per-job broadcast channel. When a subscriber
/// falls this far behind it receives `RecvError::Lagged` and the SSE handler
/// re-backfills from disk to recover the gap.
const CHANNEL_CAPACITY: usize = 1024;

/// A live log event for a single build job. Cloneable so it can fan out to
/// every subscriber of the job's broadcast channel.
#[derive(Clone, Debug)]
pub enum LogEvent {
    /// A freshly appended log chunk. `end_offset` is the file size *after*
    /// this append, used by subscribers to dedupe against their backfill.
    Chunk {
        source: String,
        bytes: Vec<u8>,
        end_offset: u64,
    },
    /// The job's worker session finished; no more chunks will arrive.
    Complete,
}

/// Per-job fan-out of live log chunks. The daemon publishes from the worker
/// socket handler; SSE handlers subscribe to stream live output.
#[derive(Clone, Default)]
pub struct LogBroadcaster {
    channels: Arc<Mutex<HashMap<Uuid, broadcast::Sender<LogEvent>>>>,
}

impl LogBroadcaster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to a job's live log events, creating the channel lazily so a
    /// subscriber that connects before the worker can still receive output.
    pub fn subscribe(&self, job_id: Uuid) -> broadcast::Receiver<LogEvent> {
        let mut channels = self
            .channels
            .lock()
            .expect("log broadcaster mutex poisoned");
        let sender = channels
            .entry(job_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);
        sender.subscribe()
    }

    /// Publish a freshly appended chunk. No-op if no channel exists yet
    /// (i.e. nobody is listening); chunks are durable on disk regardless.
    pub fn publish_chunk(&self, job_id: Uuid, source: &str, bytes: Vec<u8>, end_offset: u64) {
        let sender = {
            let channels = self
                .channels
                .lock()
                .expect("log broadcaster mutex poisoned");
            channels.get(&job_id).cloned()
        };
        if let Some(sender) = sender {
            let _ = sender.send(LogEvent::Chunk {
                source: source.to_string(),
                bytes,
                end_offset,
            });
        }
    }

    /// Signal that the job is finished, then drop its channel so memory is
    /// reclaimed. Receivers observe `Complete`, or a closed channel if they
    /// were created after this call.
    pub fn publish_complete(&self, job_id: Uuid) {
        let sender = {
            let mut channels = self
                .channels
                .lock()
                .expect("log broadcaster mutex poisoned");
            channels.remove(&job_id)
        };
        if let Some(sender) = sender {
            let _ = sender.send(LogEvent::Complete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::TryRecvError;

    #[tokio::test]
    async fn publish_chunk_reaches_subscriber() {
        let broadcaster = LogBroadcaster::new();
        let job = Uuid::now_v7();
        let mut rx = broadcaster.subscribe(job);

        broadcaster.publish_chunk(job, "worker.log", b"hello".to_vec(), 5);

        match rx.recv().await.expect("recv") {
            LogEvent::Chunk {
                source,
                bytes,
                end_offset,
            } => {
                assert_eq!(source, "worker.log");
                assert_eq!(bytes, b"hello");
                assert_eq!(end_offset, 5);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn fan_out_to_multiple_subscribers() {
        let broadcaster = LogBroadcaster::new();
        let job = Uuid::now_v7();
        let mut rx1 = broadcaster.subscribe(job);
        let mut rx2 = broadcaster.subscribe(job);

        broadcaster.publish_chunk(job, "worker.log", b"x".to_vec(), 1);

        assert!(matches!(
            rx1.recv().await.expect("rx1"),
            LogEvent::Chunk { .. }
        ));
        assert!(matches!(
            rx2.recv().await.expect("rx2"),
            LogEvent::Chunk { .. }
        ));
    }

    #[tokio::test]
    async fn complete_then_channel_dropped() {
        let broadcaster = LogBroadcaster::new();
        let job = Uuid::now_v7();
        let mut rx = broadcaster.subscribe(job);

        broadcaster.publish_complete(job);

        assert!(matches!(rx.recv().await.expect("recv"), LogEvent::Complete));
        // Channel removed: sender dropped, subsequent recv reports closed.
        assert!(matches!(
            rx.recv().await,
            Err(broadcast::error::RecvError::Closed)
        ));

        // A subscription created after completion sees an empty closed channel.
        let mut late = broadcaster.subscribe(job);
        assert!(matches!(late.try_recv(), Err(TryRecvError::Empty)));
    }

    #[tokio::test]
    async fn publish_without_subscriber_is_noop() {
        let broadcaster = LogBroadcaster::new();
        let job = Uuid::now_v7();
        // Should not panic even though nobody subscribed.
        broadcaster.publish_chunk(job, "worker.log", b"data".to_vec(), 4);
        broadcaster.publish_complete(job);
    }
}
