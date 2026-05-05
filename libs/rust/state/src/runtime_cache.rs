use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use redis::{AsyncCommands, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use synforge_core::{
    api::{JobResourceUsageSample, MockChrootListResponse},
    config::DaemonConfig,
    model::now_utc,
};
use tracing::warn;
use uuid::Uuid;

const REDIS_MAX_ATTEMPTS: u32 = 3;
const REDIS_RETRY_BASE_DELAY: Duration = Duration::from_millis(50);

/// Retry idempotent Redis operations on transient errors. Mutations
/// must not call this — only reads (GET/HGET/HGETALL/PING). The
/// underlying ConnectionManager already auto-reconnects on dropped
/// connections, but a single op that lands during the reconnect
/// still surfaces an `IoError` to us; retrying covers that gap.
async fn retry_redis_read<T, F, Fut>(label: &'static str, mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = redis::RedisResult<T>>,
{
    let mut delay = REDIS_RETRY_BASE_DELAY;
    for attempt in 1..=REDIS_MAX_ATTEMPTS {
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < REDIS_MAX_ATTEMPTS && is_transient_redis_error(&error) => {
                warn!(
                    label,
                    attempt,
                    delay_ms = delay.as_millis() as u64,
                    error = %error,
                    "redis read failed; retrying"
                );
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("retry loop exited without returning")
}

fn is_transient_redis_error(error: &redis::RedisError) -> bool {
    use redis::{ErrorKind, ServerErrorKind};
    matches!(
        error.kind(),
        ErrorKind::Io
            | ErrorKind::Server(
                ServerErrorKind::TryAgain
                    | ServerErrorKind::BusyLoading
                    | ServerErrorKind::ClusterDown
                    | ServerErrorKind::MasterDown,
            )
    )
}

pub const UI_SESSION_TTL_SECONDS: u64 = 60 * 60 * 24 * 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedMockChrootEntry {
    pub worker_image: String,
    pub fetched_at_unix_seconds: i64,
    pub response: MockChrootListResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiSessionRecord {
    pub token: String,
    pub user_id: Uuid,
    pub issued_at: i64,
    pub expires_at: i64,
}

#[derive(Clone)]
pub struct RuntimeCache {
    connection: ConnectionManager,
    key_prefix: String,
}

impl RuntimeCache {
    pub async fn new(config: &DaemonConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self {
            connection,
            key_prefix: config.redis_key_prefix.clone(),
        })
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        let pong: String = retry_redis_read("ping", || {
            let mut connection = self.connection.clone();
            async move { redis::cmd("PING").query_async(&mut connection).await }
        })
        .await?;
        if pong != "PONG" {
            anyhow::bail!("unexpected redis ping response: {pong}");
        }
        Ok(())
    }

    pub async fn create_ui_session(&self, user_id: Uuid) -> anyhow::Result<UiSessionRecord> {
        let token = generate_session_token()?;
        let issued_at = now_utc().unix_timestamp();
        let expires_at = issued_at + UI_SESSION_TTL_SECONDS as i64;
        let session = UiSessionRecord {
            token,
            user_id,
            issued_at,
            expires_at,
        };
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(&session)?;
        let _: () = connection
            .set_ex(
                self.ui_session_key(&session.token),
                value,
                UI_SESSION_TTL_SECONDS,
            )
            .await?;
        Ok(session)
    }

    pub async fn get_ui_session(&self, token: &str) -> anyhow::Result<Option<UiSessionRecord>> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let key = self.ui_session_key(token);
        let value: Option<String> = retry_redis_read("get_ui_session", || {
            let mut connection = self.connection.clone();
            let key = key.clone();
            async move { connection.get(key).await }
        })
        .await?;
        let session = value
            .map(|value| {
                serde_json::from_str::<UiSessionRecord>(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached UI session JSON: {error}"))
            })
            .transpose()?;
        let Some(session) = session else {
            return Ok(None);
        };
        if session.expires_at <= now_utc().unix_timestamp() {
            self.delete_ui_session(token).await?;
            return Ok(None);
        }
        Ok(Some(session))
    }

    pub async fn delete_ui_session(&self, token: &str) -> anyhow::Result<()> {
        if token.trim().is_empty() {
            return Ok(());
        }
        let mut connection = self.connection.clone();
        let _: usize = connection.del(self.ui_session_key(token)).await?;
        Ok(())
    }

    pub async fn get_mock_chroot_entry(
        &self,
        worker_image: &str,
    ) -> anyhow::Result<Option<CachedMockChrootEntry>> {
        let key = self.mock_chroots_key(worker_image);
        let value: Option<String> = retry_redis_read("get_mock_chroot_entry", || {
            let mut connection = self.connection.clone();
            let key = key.clone();
            async move { connection.get(key).await }
        })
        .await?;
        value
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached mock chroot JSON: {error}"))
            })
            .transpose()
    }

    pub async fn set_mock_chroot_entry(
        &self,
        worker_image: &str,
        ttl_seconds: u64,
        entry: &CachedMockChrootEntry,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(entry)?;
        let _: () = connection
            .set_ex(self.mock_chroots_key(worker_image), value, ttl_seconds)
            .await?;
        Ok(())
    }

    pub async fn list_job_usage_samples(&self) -> anyhow::Result<Vec<JobResourceUsageSample>> {
        let key = self.job_usage_key();
        let values: HashMap<String, String> = retry_redis_read("list_job_usage_samples", || {
            let mut connection = self.connection.clone();
            let key = key.clone();
            async move { connection.hgetall(key).await }
        })
        .await?;
        values
            .into_values()
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached job usage JSON: {error}"))
            })
            .collect()
    }

    pub async fn get_job_usage_sample(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Option<JobResourceUsageSample>> {
        let key = self.job_usage_key();
        let field = job_id.to_string();
        let value: Option<String> = retry_redis_read("get_job_usage_sample", || {
            let mut connection = self.connection.clone();
            let key = key.clone();
            let field = field.clone();
            async move { connection.hget(key, field).await }
        })
        .await?;
        value
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached job usage JSON: {error}"))
            })
            .transpose()
    }

    pub async fn set_job_usage_sample(
        &self,
        sample: &JobResourceUsageSample,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(sample)?;
        let _: usize = connection
            .hset(self.job_usage_key(), sample.job_id.to_string(), value)
            .await?;
        Ok(())
    }

    pub async fn remove_job_usage_sample(&self, job_id: Uuid) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let _: usize = connection
            .hdel(self.job_usage_key(), job_id.to_string())
            .await?;
        Ok(())
    }

    fn mock_chroots_key(&self, worker_image: &str) -> String {
        format!("{}:mock-chroots:{worker_image}", self.key_prefix)
    }

    fn job_usage_key(&self) -> String {
        format!("{}:job-usage", self.key_prefix)
    }

    fn ui_session_key(&self, token: &str) -> String {
        format!("{}:ui-session:{token}", self.key_prefix)
    }
}

fn generate_session_token() -> anyhow::Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| anyhow::anyhow!("failed to read OS entropy: {}", error))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}
