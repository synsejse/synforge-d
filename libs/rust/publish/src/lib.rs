//! Repository publication infrastructure for the rewrite target architecture.

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
