use std::collections::BTreeMap;

use crate::DieselStore;
use serde_json::Value;

#[derive(Clone)]
pub struct PostgresRuntimeSettingsStore {
    store: DieselStore,
}

impl PostgresRuntimeSettingsStore {
    pub fn new(store: DieselStore) -> Self {
        Self { store }
    }

    pub async fn list(&self) -> anyhow::Result<BTreeMap<String, Value>> {
        self.store.list_runtime_settings().await
    }

    pub async fn upsert(&self, settings: BTreeMap<String, Value>) -> anyhow::Result<()> {
        self.store.upsert_runtime_settings(settings).await
    }
}
