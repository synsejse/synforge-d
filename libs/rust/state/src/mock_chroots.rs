use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MockChrootCacheEntry {
    pub worker_image: String,
    pub fetched_at_unix_seconds: i64,
    pub chroots: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MockChrootCacheSnapshot {
    pub entry: Option<MockChrootCacheEntry>,
    pub hit_count: u64,
    pub miss_count: u64,
    pub stale_served_count: u64,
}

#[derive(Clone, Default)]
pub struct MockChrootCache {
    inner: Arc<Mutex<MockChrootCacheSnapshot>>,
}

impl MockChrootCache {
    pub async fn try_get_fresh(
        &self,
        worker_image: &str,
        now_unix_seconds: i64,
        ttl_seconds: u64,
    ) -> Option<Vec<String>> {
        let mut cache = self.inner.lock().await;
        let chroots = cache.entry.as_ref().and_then(|entry| {
            if entry.worker_image == worker_image
                && now_unix_seconds.saturating_sub(entry.fetched_at_unix_seconds)
                    < ttl_seconds as i64
            {
                Some(entry.chroots.clone())
            } else {
                None
            }
        });
        if chroots.is_some() {
            cache.hit_count = cache.hit_count.saturating_add(1);
        }
        chroots
    }

    pub async fn install_cached_entry(&self, entry: MockChrootCacheEntry) -> Vec<String> {
        let chroots = entry.chroots.clone();
        let mut cache = self.inner.lock().await;
        cache.hit_count = cache.hit_count.saturating_add(1);
        cache.entry = Some(entry);
        chroots
    }

    pub async fn record_miss(&self) -> Option<MockChrootCacheEntry> {
        let mut cache = self.inner.lock().await;
        cache.miss_count = cache.miss_count.saturating_add(1);
        cache.entry.clone()
    }

    pub async fn install_fresh_entry(&self, entry: MockChrootCacheEntry) {
        let mut cache = self.inner.lock().await;
        cache.entry = Some(entry);
    }

    pub async fn record_stale_served(&self) {
        let mut cache = self.inner.lock().await;
        cache.stale_served_count = cache.stale_served_count.saturating_add(1);
    }

    pub async fn snapshot(&self) -> MockChrootCacheSnapshot {
        self.inner.lock().await.clone()
    }
}
