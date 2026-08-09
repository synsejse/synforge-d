use synforge_core::{
    api::{CacheStatsResponse, GitMirrorCacheStats, MockChrootCacheStats},
    model::{format_timestamp, now_utc},
};
use synforge_database::JobStore;
use synforge_state::MockChrootCacheSnapshot;
use time::OffsetDateTime;

use super::SynforgeService;

impl SynforgeService {
    pub async fn get_cache_stats(&self) -> anyhow::Result<CacheStatsResponse> {
        let now_unix_seconds = now_utc().unix_timestamp();
        let mock_chroot_cache = build_mock_chroot_cache_stats(
            &self.mock_chroot_cache.snapshot().await,
            now_unix_seconds,
            self.config.mock_chroot_cache_ttl_seconds,
        );

        let git_stats = self.registry.git_cache_stats().await?;
        let git_mirror_cache = GitMirrorCacheStats {
            mirror_root: git_stats.mirror_root,
            refresh_ttl_seconds: git_stats.refresh_ttl_seconds,
            max_unused_seconds: git_stats.max_unused_seconds,
            tracked_mirrors: git_stats.tracked_mirrors,
            stale_mirrors: git_stats.stale_mirrors,
            mirror_directories: git_stats.mirror_directories,
            latest_fetched_at: git_stats.latest_fetched_at.and_then(format_unix_timestamp),
            latest_used_at: git_stats.latest_used_at.and_then(format_unix_timestamp),
        };
        let compiler_cache = self.store.get_workspace_ccache_stats().await?;

        Ok(CacheStatsResponse {
            collected_at: format_timestamp(now_utc()),
            mock_chroot_cache,
            git_mirror_cache,
            compiler_cache,
        })
    }
}

fn build_mock_chroot_cache_stats(
    cache: &MockChrootCacheSnapshot,
    now_unix_seconds: i64,
    ttl_seconds: u64,
) -> MockChrootCacheStats {
    let (cached_chroot_count, age_seconds, worker_image, last_refresh_at) =
        if let Some(entry) = cache.entry.as_ref() {
            (
                entry.chroots.len(),
                now_unix_seconds
                    .checked_sub(entry.fetched_at_unix_seconds)
                    .map(|value| value as u64),
                Some(entry.worker_image.clone()),
                format_unix_timestamp(entry.fetched_at_unix_seconds),
            )
        } else {
            (0, None, None, None)
        };
    MockChrootCacheStats {
        ttl_seconds,
        hit_count: cache.hit_count,
        miss_count: cache.miss_count,
        stale_served_count: cache.stale_served_count,
        cached_chroot_count,
        age_seconds,
        worker_image,
        last_refresh_at,
    }
}

fn format_unix_timestamp(value: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(value)
        .ok()
        .map(format_timestamp)
}
