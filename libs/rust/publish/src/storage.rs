use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{RepoFileStorage, RuntimeRepoAdapter};

#[async_trait]
impl RepoFileStorage for RuntimeRepoAdapter {
    async fn restore_repo_file(
        &self,
        _relative_repo_path: &Path,
        _destination_path: &Path,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }
}

impl RuntimeRepoAdapter {
    pub async fn sync_repo_tree(&self, _repo_root: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn sanitize_relative_path(path: &str) -> PathBuf {
    Path::new(path)
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(PathBuf::from(part)),
            _ => None,
        })
        .fold(PathBuf::new(), |mut acc, part| {
            acc.push(part);
            acc
        })
}
