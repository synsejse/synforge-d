mod artifacts;
mod commands;
mod keyring;
mod repo_metadata;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSigningKey {
    pub key_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSigningStatus {
    pub enabled: bool,
    pub configured_key_id: Option<String>,
    pub keyring_dir: PathBuf,
    pub key_present: bool,
    pub active_fingerprint: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RepoSigningManager;
