mod build;
mod mock;
mod tooling;

use async_trait::async_trait;
use synforge_core::{
    model::{WorkerBuildPayload, WorkerBuildResult, WorkerJobPayload},
    package::PackageDefinition,
};

use self::build::execute_spec_build;
use self::mock::{build_source_rpm, run_mock_build};
use self::tooling::{fetch_spec_sources, prepare_build_tooling, prepare_topdir};
use crate::protocol::WorkerTransportHandle;

#[async_trait]
pub trait BuildExecutor: Send + Sync {
    async fn execute_build(
        &self,
        package: &PackageDefinition,
        payload: &WorkerJobPayload,
        build: &WorkerBuildPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerBuildResult>;
}

#[derive(Debug, Default, Clone)]
pub struct RpmBuildExecutor;

#[async_trait]
impl BuildExecutor for RpmBuildExecutor {
    async fn execute_build(
        &self,
        package: &PackageDefinition,
        payload: &WorkerJobPayload,
        build: &WorkerBuildPayload,
        transport: Option<WorkerTransportHandle>,
    ) -> anyhow::Result<WorkerBuildResult> {
        execute_spec_build(package, payload, build, transport).await
    }
}
