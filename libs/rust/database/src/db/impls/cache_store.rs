use super::super::*;

#[async_trait]
impl GitCacheStore for DieselStore {
    async fn get_git_mirror_cache_state(
        &self,
        mirror_key: &str,
    ) -> anyhow::Result<Option<GitMirrorCacheState>> {
        cache::get_git_mirror_cache_state(self, mirror_key).await
    }

    async fn upsert_git_mirror_cache_state(
        &self,
        mirror_key: &str,
        repo_url: &str,
        last_fetched_at: i64,
        last_used_at: i64,
    ) -> anyhow::Result<()> {
        cache::upsert_git_mirror_cache_state(
            self,
            mirror_key,
            repo_url,
            last_fetched_at,
            last_used_at,
        )
        .await
    }

    async fn list_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<Vec<GitMirrorCacheState>> {
        cache::list_stale_git_mirror_cache_states(self, last_used_before_or_equal).await
    }

    async fn delete_git_mirror_cache_state(&self, mirror_key: &str) -> anyhow::Result<()> {
        cache::delete_git_mirror_cache_state(self, mirror_key).await
    }

    async fn count_git_mirror_cache_states(&self) -> anyhow::Result<u64> {
        cache::count_git_mirror_cache_states(self).await
    }

    async fn count_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<u64> {
        cache::count_stale_git_mirror_cache_states(self, last_used_before_or_equal).await
    }

    async fn latest_git_mirror_cache_timestamps(
        &self,
    ) -> anyhow::Result<(Option<i64>, Option<i64>)> {
        cache::latest_git_mirror_cache_timestamps(self).await
    }
}
