use std::collections::HashMap;

use redis::{AsyncCommands, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use synforge_core::{
    api::{JobResourceUsageSample, MockChrootListResponse},
    config::DaemonConfig,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct CachedMockChrootEntry {
    pub(super) worker_image: String,
    pub(super) fetched_at_unix_seconds: i64,
    pub(super) response: MockChrootListResponse,
}

#[derive(Clone)]
pub(crate) struct RuntimeCache {
    connection: ConnectionManager,
    key_prefix: String,
}

impl RuntimeCache {
    pub(super) async fn new(config: &DaemonConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self {
            connection,
            key_prefix: config.redis_key_prefix.clone(),
        })
    }

    pub(super) async fn health_check(&self) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let pong: String = redis::cmd("PING").query_async(&mut connection).await?;
        if pong != "PONG" {
            anyhow::bail!("unexpected redis ping response: {pong}");
        }
        Ok(())
    }

    pub(super) async fn get_mock_chroot_entry(
        &self,
        worker_image: &str,
    ) -> anyhow::Result<Option<CachedMockChrootEntry>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.mock_chroots_key(worker_image)).await?;
        value
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached mock chroot JSON: {error}"))
            })
            .transpose()
    }

    pub(super) async fn set_mock_chroot_entry(
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

    pub(super) async fn list_job_usage_samples(
        &self,
    ) -> anyhow::Result<Vec<JobResourceUsageSample>> {
        let mut connection = self.connection.clone();
        let values: HashMap<String, String> = connection.hgetall(self.job_usage_key()).await?;
        values
            .into_values()
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached job usage JSON: {error}"))
            })
            .collect()
    }

    pub(super) async fn get_job_usage_sample(
        &self,
        job_id: Uuid,
    ) -> anyhow::Result<Option<JobResourceUsageSample>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection
            .hget(self.job_usage_key(), job_id.to_string())
            .await?;
        value
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|error| anyhow::anyhow!("invalid cached job usage JSON: {error}"))
            })
            .transpose()
    }

    pub(super) async fn set_job_usage_sample(
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

    pub(super) async fn remove_job_usage_sample(&self, job_id: Uuid) -> anyhow::Result<()> {
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
}
