use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptionsBuilder, LogsOptionsBuilder};
use futures_util::StreamExt;
use synforge_core::{model::now_utc, package::parse_mock_chroot};
use synforge_state::{CachedMockChrootEntry, MockChrootCache, MockChrootCacheEntry, RuntimeCache};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct MockChrootService {
    runtime_cache: RuntimeCache,
    memory_cache: MockChrootCache,
}

impl MockChrootService {
    pub fn new(runtime_cache: RuntimeCache, memory_cache: MockChrootCache) -> Self {
        Self {
            runtime_cache,
            memory_cache,
        }
    }

    pub async fn list_mock_chroots(
        &self,
        worker_image: &str,
        ttl_seconds: u64,
    ) -> anyhow::Result<Vec<String>> {
        let now_unix_seconds = now_utc().unix_timestamp();
        if let Some(chroots) = self
            .memory_cache
            .try_get_fresh(worker_image, now_unix_seconds, ttl_seconds)
            .await
        {
            return Ok(chroots);
        }
        if let Some(entry) = self
            .runtime_cache
            .get_mock_chroot_entry(worker_image)
            .await?
            .filter(|entry| {
                now_unix_seconds.saturating_sub(entry.fetched_at_unix_seconds) < ttl_seconds as i64
            })
        {
            return Ok(self
                .memory_cache
                .install_cached_entry(MockChrootCacheEntry {
                    worker_image: entry.worker_image.clone(),
                    fetched_at_unix_seconds: entry.fetched_at_unix_seconds,
                    chroots: entry.chroots.clone(),
                })
                .await);
        }

        let stale = self.memory_cache.record_miss().await;
        match load_mock_chroots_uncached(worker_image).await {
            Ok(chroots) => {
                let entry = CachedMockChrootEntry {
                    worker_image: worker_image.to_string(),
                    fetched_at_unix_seconds: now_unix_seconds,
                    chroots: chroots.clone(),
                };
                self.runtime_cache
                    .set_mock_chroot_entry(worker_image, ttl_seconds, &entry)
                    .await?;
                self.memory_cache
                    .install_fresh_entry(MockChrootCacheEntry {
                        worker_image: entry.worker_image,
                        fetched_at_unix_seconds: entry.fetched_at_unix_seconds,
                        chroots: chroots.clone(),
                    })
                    .await;
                Ok(chroots)
            }
            Err(error) => {
                if let Some(entry) = stale
                    && entry.worker_image == worker_image
                {
                    self.memory_cache.record_stale_served().await;
                    warn!(
                        worker_image,
                        error = %error,
                        stale_count = entry.chroots.len(),
                        "failed to refresh mock chroot cache; serving stale value"
                    );
                    return Ok(entry.chroots);
                }
                Err(error)
            }
        }
    }
}

async fn load_mock_chroots_uncached(worker_image: &str) -> anyhow::Result<Vec<String>> {
    let docker = Docker::connect_with_local_defaults()?;
    let container_name = format!("synforge-mock-chroots-{}", Uuid::now_v7());
    info!(
        container_name = %container_name,
        worker_image,
        "listing mock chroots via worker image"
    );
    let container = docker
        .create_container(
            Some(
                CreateContainerOptionsBuilder::default()
                    .name(container_name.as_str())
                    .build(),
            ),
            ContainerCreateBody {
                image: Some(worker_image.to_string()),
                entrypoint: Some(vec!["bash".to_string(), "-lc".to_string()]),
                cmd: Some(vec!["mock --list-chroots".to_string()]),
                host_config: Some(HostConfig {
                    auto_remove: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let container_id = container.id;
    info!(container_id = %container_id, "starting mock chroot listing container");
    docker
        .start_container(
            &container_id,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await?;

    let mut stdout = String::new();
    let mut logs = docker.logs(
        &container_id,
        Some(
            LogsOptionsBuilder::default()
                .follow(true)
                .stdout(true)
                .stderr(false)
                .timestamps(false)
                .tail("all")
                .build(),
        ),
    );
    while let Some(item) = logs.next().await {
        stdout.push_str(&String::from_utf8_lossy(item?.into_bytes().as_ref()));
    }

    let mut wait = docker.wait_container(
        &container_id,
        None::<bollard::query_parameters::WaitContainerOptions>,
    );
    while let Some(next) = wait.next().await {
        next?;
    }
    let mut chroots = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| name.starts_with("fedora-"))
        .filter(|name| parse_mock_chroot(name).is_some())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    chroots.sort();
    chroots.dedup();
    info!(
        container_id = %container_id,
        chroot_count = chroots.len(),
        "mock chroot listing completed"
    );
    Ok(chroots)
}
