use super::super::*;

#[async_trait]
impl PackageStore for DieselStore {
    async fn list_packages(
        &self,
        limit: usize,
        offset: usize,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<PackageResponse>> {
        package::list_packages(self, limit, offset, search, enabled).await
    }

    async fn count_packages(
        &self,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<u64> {
        package::count_packages(self, search, enabled).await
    }

    async fn get_package(&self, package_name: &str) -> anyhow::Result<Option<PackageResponse>> {
        package::get_package(self, package_name).await
    }

    async fn upsert_package(&self, package: &PackageDefinition) -> anyhow::Result<()> {
        package::upsert_package(self, package).await
    }

    async fn remove_package(&self, package_name: &str) -> anyhow::Result<()> {
        package::remove_package(self, package_name).await
    }
}
