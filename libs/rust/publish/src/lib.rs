//! Repository publication infrastructure for the rewrite target architecture.

use strict_path::{PathBoundary, StrictPath};
use synforge_core::{config::DaemonConfig, model::BuildArtifact};
use uuid::Uuid;

pub mod repo_manager;
mod service;
pub mod signing;

pub use repo_manager::FileRepoManager;
pub use service::{
    RepoArtifactCatalog, RepoInventoryReader, RepoService, RepoSigningCommandRunner,
    RepoSigningConfigLoader, RepoSigningInspector, RepoSigningKeyIdentity,
    RepoSigningProgressReader, RepoSigningProgressWriter, RepoSigningSettingsUpdate,
    RepoSigningSettingsWriter, RepoSummaryReader,
};
pub use signing::{ImportedSigningKey, RepoSigningManager, RepoSigningStatus};

pub(crate) fn resolve_job_artifact_path(
    config: &DaemonConfig,
    job_id: Uuid,
    artifact: &BuildArtifact,
) -> anyhow::Result<StrictPath> {
    let artifact_root = config.runtime_paths().job_artifacts_dir(job_id);
    let boundary = PathBoundary::try_new(artifact_root)?;
    Ok(boundary.strict_join(artifact.storage_path()?)?)
}
