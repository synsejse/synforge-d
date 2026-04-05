use tokio::io::{AsyncReadExt, AsyncSeekExt};
use uuid::Uuid;

use super::SynforgeService;
use anyhow::Context;
use synforge_core::{
    api::{LogChunkResponse, LogManifestResponse, LogMetaResponse, LogSource, LogSourceType},
    error::SynforgeError,
};
use crate::db::JobStore;

impl SynforgeService {
    async fn resolve_job_log_path(
        &self,
        job_id: Uuid,
        source: &str,
    ) -> anyhow::Result<std::path::PathBuf> {
        let row = self
            .store
            .get_build_log_for_job_source(job_id, source)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(SynforgeError::NotFound(format!(
                    "log source {} for job {}",
                    source, job_id
                )))
            })?;
        let path = self.config.runtime_paths().job_log_path(job_id, &row.file);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
        Err(anyhow::anyhow!(SynforgeError::NotFound(
            path.display().to_string()
        )))
    }

    pub async fn get_job_log_chunk(
        &self,
        job_id: Uuid,
        source: String,
        cursor: Option<u64>,
        offset: Option<i64>,
        limit: Option<usize>,
    ) -> anyhow::Result<LogChunkResponse> {
        let path = self.resolve_job_log_path(job_id, &source).await?;

        let mut file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("failed to open {}", path.display()))?;

        let file_size = file.metadata().await?.len();
        let max_len = limit.unwrap_or(64 * 1024).clamp(1024, 512 * 1024) as u64;
        let start = ((cursor.unwrap_or(0).min(file_size) as i128) + offset.unwrap_or(0) as i128)
            .clamp(0, file_size as i128) as u64;
        let read_len = max_len.min(file_size.saturating_sub(start));

        if read_len == 0 {
            return Ok(LogChunkResponse {
                job_id,
                source,
                contents: String::new(),
                start_line: count_lines_before(&path, start).await?,
                cursor: start,
                complete: true,
            });
        }

        file.seek(std::io::SeekFrom::Start(start)).await?;
        let mut buffer = vec![0u8; read_len as usize];
        let bytes_read = file.read(&mut buffer).await?;
        buffer.truncate(bytes_read);

        // Find UTF-8 safe boundary to avoid splitting multi-byte characters
        let safe_len = find_utf8_boundary(&buffer);
        buffer.truncate(safe_len);

        let contents = String::from_utf8_lossy(&buffer).into_owned();
        let start_line = count_lines_before(&path, start).await?;
        let new_cursor = start + safe_len as u64;
        let complete = new_cursor >= file_size;

        Ok(LogChunkResponse {
            job_id,
            source,
            contents,
            start_line,
            cursor: new_cursor,
            complete,
        })
    }

    pub async fn get_job_log_meta(
        &self,
        job_id: Uuid,
        source: String,
    ) -> anyhow::Result<LogMetaResponse> {
        let path = self.resolve_job_log_path(job_id, &source).await?;
        let file_size = tokio::fs::metadata(&path)
            .await
            .with_context(|| format!("failed to stat {}", path.display()))?
            .len();
        Ok(LogMetaResponse {
            job_id,
            source,
            file_size,
            max_cursor: file_size,
        })
    }

    pub async fn get_job_log_manifest(&self, job_id: Uuid) -> anyhow::Result<LogManifestResponse> {
        let mut sources = Vec::new();
        let db_logs = self.store.list_build_logs_for_job(job_id).await?;

        for row in db_logs {
            let log_path = self.config.runtime_paths().job_log_path(job_id, &row.file);
            if let Ok(meta) = tokio::fs::metadata(&log_path).await {
                sources.push(LogSource {
                    file: row.file,
                    size: meta.len(),
                    source_type: LogSourceType::Raw,
                });
            }
        }

        Ok(LogManifestResponse { job_id, sources })
    }
}

/// Find a safe UTF-8 boundary in a byte slice.
/// Returns the largest index <= buffer.len() that doesn't split a UTF-8 character.
fn find_utf8_boundary(buffer: &[u8]) -> usize {
    if buffer.is_empty() {
        return 0;
    }

    // Start from the end and work backwards to find a valid boundary
    let mut end = buffer.len();

    // Check if we're in the middle of a UTF-8 sequence
    // UTF-8 continuation bytes have the pattern 10xxxxxx (0x80-0xBF)
    while end > 0 {
        let byte = buffer[end - 1];

        // If this byte is ASCII or a UTF-8 start byte, we're at a valid boundary
        if byte < 0x80 {
            // ASCII byte - valid boundary
            break;
        } else if byte >= 0xC0 {
            // UTF-8 start byte (110xxxxx, 1110xxxx, or 11110xxx)
            // Check if there's enough room for the full character
            let char_len = if byte >= 0xF0 {
                4
            } else if byte >= 0xE0 {
                3
            } else {
                2
            };

            if end + char_len - 1 <= buffer.len() {
                // Full character fits, include it
                break;
            } else {
                // Character is truncated, exclude it
                end -= 1;
            }
        } else {
            // Continuation byte (10xxxxxx) - go back to find start
            end -= 1;
        }
    }

    end
}

async fn count_lines_before(path: &std::path::Path, end_offset: u64) -> anyhow::Result<u64> {
    if end_offset == 0 {
        return Ok(1);
    }

    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("failed to open {}", path.display()))?;
    file.seek(std::io::SeekFrom::Start(0)).await?;

    let mut remaining = end_offset;
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut line_count = 0_u64;

    while remaining > 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        let bytes_read = file.read(&mut buffer[..read_len]).await?;
        if bytes_read == 0 {
            break;
        }
        line_count += buffer[..bytes_read]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count() as u64;
        remaining = remaining.saturating_sub(bytes_read as u64);
    }

    Ok(line_count + 1)
}
