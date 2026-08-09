use std::collections::HashMap;
use std::convert::Infallible;

use super::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use axum::routing::get;
use futures_util::Stream;
use synforge_core::api::LogManifestResponse;
use synforge_worker_host::LogEvent;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use uuid::Uuid;

use crate::service::find_utf8_boundary;

/// Backfill read granularity. Each disk read produces at most one `append`
/// event of this size (after trimming to a UTF-8 boundary).
const BACKFILL_CHUNK_BYTES: usize = 64 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs/{id}/logs", get(get_job_log_manifest))
        .route("/jobs/{id}/logs/stream", get(stream_job_logs))
        .route("/sync/operations/{id}/logs", get(get_sync_log_manifest))
        .route("/sync/operations/{id}/logs/stream", get(stream_sync_logs))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get available log sources for a job", body = LogManifestResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_job_log_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LogManifestResponse>, AppError> {
    Ok(Json(state.service.get_job_log_manifest(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/operations/{id}/logs",
    tag = "Logs",
    params(("id" = Uuid, Path, description = "Sync operation identifier")),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Get available log sources for a sync", body = LogManifestResponse),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn get_sync_log_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LogManifestResponse>, AppError> {
    Ok(Json(state.service.get_sync_log_manifest(id).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}/logs/stream",
    tag = "Logs",
    params(
        ("id" = Uuid, Path, description = "Job identifier")
    ),
    security(("session_auth" = [])),
    responses(
        (
            status = 200,
            description = "Server-sent event stream of job logs. Emits a `manifest` event \
                           with the available sources, `append` events carrying log text, \
                           and a final `complete` event. Each connection replays the full \
                           log from the start, then tails live output until the build ends.",
            content_type = "text/event-stream"
        ),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn stream_job_logs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    stream_logs(state, id, LogOwner::Build).await
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/operations/{id}/logs/stream",
    tag = "Logs",
    params(("id" = Uuid, Path, description = "Sync operation identifier")),
    security(("session_auth" = [])),
    responses(
        (status = 200, description = "Server-sent event stream of sync logs", content_type = "text/event-stream"),
        (status = 401, body = synforge_core::api::ApiError),
        (status = 404, body = synforge_core::api::ApiError)
    )
)]
pub(super) async fn stream_sync_logs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    stream_logs(state, id, LogOwner::Sync).await
}

#[derive(Clone, Copy)]
enum LogOwner {
    Build,
    Sync,
}

async fn stream_logs(
    state: AppState,
    id: Uuid,
    owner: LogOwner,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let service = state.service.clone();
    let broadcaster = service.log_broadcaster();

    // 1. Subscribe BEFORE resolving / terminal-checking the job. This buffers
    //    chunks arriving during backfill, AND closes a completion race: if the
    //    job finishes between the terminal check and subscribe, publish_complete
    //    would have removed the channel and we'd block forever on a fresh one.
    //    Subscribing first guarantees we either receive Complete or read
    //    terminal_at_start = true (status is set before publish_complete).
    let mut rx = broadcaster.subscribe(id);

    // 2. Resolve the job (unknown -> 404) and capture whether it had already
    //    finished, so a stream opened on a completed job ends instead of hanging.
    let terminal_at_start = match owner {
        LogOwner::Build => service.job_is_terminal(id).await?,
        LogOwner::Sync => service.sync_is_terminal(id).await?,
    };

    let stream = async_stream::stream! {
        // 3. Manifest event lists the currently-known sources.
        let manifest = match owner {
            LogOwner::Build => service.get_job_log_manifest(id).await,
            LogOwner::Sync => service.get_sync_log_manifest(id).await,
        };
        let manifest = match manifest {
            Ok(manifest) => manifest,
            Err(_) => {
                yield Ok(complete_event());
                return;
            }
        };
        let sources: Vec<String> = manifest.sources.iter().map(|s| s.file.clone()).collect();
        yield Ok(manifest_event(&sources));

        // Per-source high-water mark: the byte offset we have emitted up to.
        // After backfill this equals the file size captured below; the live
        // loop dedupes against it so backfilled bytes are never re-sent.
        let mut offsets: HashMap<String, u64> = HashMap::new();

        // 4. Backfill each source from the start. We always full-replay on
        //    (re)connect; the client resets its buffer on each `open`, so there
        //    is no cursor/resume state to get out of sync across sources.
        for source in &sources {
            offsets.insert(source.clone(), 0);
            match backfill_source(&service, id, source, 0).await {
                Ok(events) => {
                    for (event, new_offset) in events {
                        offsets.insert(source.clone(), new_offset);
                        yield Ok(event);
                    }
                }
                Err(_) => {
                    // Missing/unreadable source: nothing to backfill, the live
                    // loop may still pick it up once chunks arrive.
                }
            }
        }

        // 6. If the job was already terminal when we started, the worker has
        //    finished and no further chunks will arrive. End now instead of
        //    blocking on a channel that will never produce.
        if terminal_at_start {
            yield Ok(complete_event());
            return;
        }

        // 5. Live: forward chunks, deduping anything already covered by backfill.
        loop {
            match rx.recv().await {
                Ok(LogEvent::Chunk { source, bytes: _, end_offset }) => {
                    let seen = offsets.get(&source).copied().unwrap_or(0);
                    if end_offset <= seen {
                        // Fully covered by backfill or a prior chunk.
                        continue;
                    }
                    // The chunk may straddle the backfill boundary. Re-read the
                    // [seen, end_offset) window from disk so the boundary lands
                    // on a UTF-8 codepoint and no bytes are dropped or doubled.
                    if let Ok(events) = backfill_source(&service, id, &source, seen).await {
                        for (event, new_offset) in events {
                            offsets.insert(source.clone(), new_offset);
                            yield Ok(event);
                        }
                    }
                }
                Ok(LogEvent::Complete) => {
                    yield Ok(complete_event());
                    return;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    yield Ok(complete_event());
                    return;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    // We dropped messages. Re-backfill every source from its
                    // last emitted offset to the current size so there are no
                    // gaps, then keep listening.
                    for source in sources.clone() {
                        let seen = offsets.get(&source).copied().unwrap_or(0);
                        if let Ok(events) = backfill_source(&service, id, &source, seen).await {
                            for (event, new_offset) in events {
                                offsets.insert(source.clone(), new_offset);
                                yield Ok(event);
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Read `[from, current_size)` of a source from disk and split it into
/// UTF-8-safe `append` events. Returns each event paired with the running end
/// offset so callers can advance their high-water mark.
async fn backfill_source(
    service: &crate::SynforgeService,
    job_id: Uuid,
    source: &str,
    from: u64,
) -> anyhow::Result<Vec<(Event, u64)>> {
    let path = service.resolve_job_log_path(job_id, source).await?;
    let mut file = tokio::fs::File::open(&path).await?;
    let size = file.metadata().await?.len();
    let mut cursor = from.min(size);
    if cursor >= size {
        return Ok(Vec::new());
    }
    file.seek(std::io::SeekFrom::Start(cursor)).await?;

    let mut events = Vec::new();
    while cursor < size {
        let remaining = (size - cursor) as usize;
        let read_len = remaining.min(BACKFILL_CHUNK_BYTES);
        let mut buffer = vec![0u8; read_len];
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        buffer.truncate(read);
        // Trim to a UTF-8 boundary, but never produce a zero-length advance
        // (which would loop forever). If the whole window is a partial
        // codepoint, emit it lossily so we still make progress.
        let mut safe_len = find_utf8_boundary(&buffer);
        if safe_len == 0 {
            safe_len = buffer.len();
        }
        buffer.truncate(safe_len);
        let text = String::from_utf8_lossy(&buffer).into_owned();
        cursor += safe_len as u64;
        events.push((append_event(source, &text, cursor), cursor));
    }
    Ok(events)
}

fn manifest_event(sources: &[String]) -> Event {
    let data = serde_json::json!({ "sources": sources });
    Event::default().event("manifest").data(data.to_string())
}

fn append_event(source: &str, text: &str, end_offset: u64) -> Event {
    let data = serde_json::json!({
        "source": source,
        "text": text,
        "end_offset": end_offset,
    });
    Event::default().event("append").data(data.to_string())
}

fn complete_event() -> Event {
    Event::default().event("complete").data("{}")
}
