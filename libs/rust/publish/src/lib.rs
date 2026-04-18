//! Repository publication infrastructure for the rewrite target architecture.

mod adapter_signing;
pub mod repo_manager;
mod service;
pub mod signing;
pub mod storage;

pub use repo_manager::FileRepoManager;
pub use service::{
    RepoArtifactCatalog, RepoFileStorage, RepoInventoryReader, RepoService,
    RepoSigningCommandRunner, RepoSigningConfigLoader, RepoSigningInspector,
    RepoSigningKeyIdentity, RepoSigningProgressReader, RepoSigningProgressWriter,
    RepoSigningSettingsUpdate, RepoSigningSettingsWriter, RepoSummaryReader,
};
pub use signing::{ImportedSigningKey, RepoSigningManager, RepoSigningStatus};
pub use storage::{JobObjectStorage, WorkerOutputStorage, WorkerOutputUpload};

#[derive(Clone)]
pub struct RuntimeRepoAdapter {
    object_storage: JobObjectStorage,
}

impl RuntimeRepoAdapter {
    pub fn new(object_storage: JobObjectStorage) -> Self {
        Self { object_storage }
    }
}
