use async_trait::async_trait;
use synforge_core::{api::PackageResponse, package::PackageDefinition};

#[async_trait]
pub trait PackageStore: Send + Sync {
    async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>>;

    async fn count_packages(
        &self,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<u64>;

    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>>;
    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()>;
    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()>;
}
