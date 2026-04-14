use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitMirrorCacheState {
    pub mirror_key: String,
    pub repo_url: String,
    pub last_fetched_at: i64,
    pub last_used_at: i64,
}

#[async_trait]
pub trait GitCacheStore: Send + Sync {
    async fn get_git_mirror_cache_state(
        &self,
        mirror_key: &str,
    ) -> anyhow::Result<Option<GitMirrorCacheState>>;

    async fn upsert_git_mirror_cache_state(
        &self,
        mirror_key: &str,
        repo_url: &str,
        last_fetched_at: i64,
        last_used_at: i64,
    ) -> anyhow::Result<()>;

    async fn list_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<Vec<GitMirrorCacheState>>;

    async fn delete_git_mirror_cache_state(&self, mirror_key: &str) -> anyhow::Result<()>;

    async fn count_git_mirror_cache_states(&self) -> anyhow::Result<u64>;

    async fn count_stale_git_mirror_cache_states(
        &self,
        last_used_before_or_equal: i64,
    ) -> anyhow::Result<u64>;

    async fn latest_git_mirror_cache_timestamps(
        &self,
    ) -> anyhow::Result<(Option<i64>, Option<i64>)>;
}
