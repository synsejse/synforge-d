use std::sync::Arc;

use synforge_core::api::{RefreshAllPackagesProgressView, RepoSigningReconcileProgressView};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ProgressState<T> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T> Default for ProgressState<T> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T: Clone> ProgressState<T> {
    pub async fn load(&self) -> Option<T> {
        self.inner.lock().await.clone()
    }
}

impl<T> ProgressState<T> {
    pub async fn save(&self, value: T) {
        let mut slot = self.inner.lock().await;
        *slot = Some(value);
    }
}

pub type RefreshAllPackagesProgressState = ProgressState<RefreshAllPackagesProgressView>;
pub type SigningReconcileProgressState = ProgressState<RepoSigningReconcileProgressView>;
