use std::path::{Path, PathBuf};

use path_clean::PathClean;
use uuid::Uuid;

use crate::package::RepoTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    repo_dir: PathBuf,
    jobs_root: PathBuf,
    cache_root: PathBuf,
    work_root: PathBuf,
    signing_root: PathBuf,
}

impl RuntimePaths {
    pub fn new(runtime_root: PathBuf) -> Self {
        let state_root = runtime_root.join("state");
        Self {
            repo_dir: runtime_root.join("repo"),
            jobs_root: state_root.join("jobs"),
            cache_root: runtime_root.join("cache"),
            work_root: runtime_root.join("work"),
            signing_root: state_root.join("signing"),
        }
    }

    pub fn repo_dir(&self) -> &Path {
        &self.repo_dir
    }

    pub fn repo_target_dir(&self, target: &RepoTarget) -> PathBuf {
        self.repo_dir.join(target.repo_subdir())
    }

    pub fn repo_target_dir_for_mock_chroot(&self, mock_chroot: &str) -> Option<PathBuf> {
        RepoTarget::from_mock_chroot(mock_chroot).map(|target| self.repo_target_dir(&target))
    }

    pub fn jobs_root(&self) -> &Path {
        &self.jobs_root
    }

    pub fn job_root(&self, job_id: Uuid) -> PathBuf {
        self.jobs_root.join(job_id.to_string())
    }

    pub fn job_artifacts_dir(&self, job_id: Uuid) -> PathBuf {
        self.job_root(job_id).join("artifacts")
    }

    pub fn job_logs_dir(&self, job_id: Uuid) -> PathBuf {
        self.job_root(job_id).join("logs")
    }

    pub fn job_log_path(&self, job_id: Uuid, file: &str) -> PathBuf {
        self.job_logs_dir(job_id).join(sanitize_relative_path(file))
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn work_root(&self) -> &Path {
        &self.work_root
    }

    pub fn signing_root(&self) -> PathBuf {
        self.signing_root.clone()
    }

    pub fn signing_public_key_path(&self, file_name: &str) -> PathBuf {
        self.signing_root.join(sanitize_relative_path(file_name))
    }

    pub fn spec_parse_workspace_dir(&self, job_id: Uuid) -> PathBuf {
        self.work_root().join("parse").join(job_id.to_string())
    }

    pub fn repo_browse_workspace_dir(&self, browse_id: Uuid) -> PathBuf {
        self.work_root().join("browse").join(browse_id.to_string())
    }

    pub fn git_cache_root(&self) -> PathBuf {
        self.cache_root().join("git")
    }

    pub fn git_mirror_root(&self) -> PathBuf {
        self.git_cache_root().join("mirrors")
    }

    pub fn git_mirror_dir(&self, mirror_key: &str) -> PathBuf {
        self.git_mirror_root()
            .join(sanitize_relative_path(mirror_key))
    }

    pub fn package_repo_dir(&self, package_name: &str, release: &str, arch: &str) -> PathBuf {
        self.repo_dir
            .join("packages")
            .join(package_name)
            .join(release)
            .join(arch)
    }
}

/// Strip `..` traversal segments and absolute roots from a relative path.
/// Returns a `PathBuf` containing only `Normal` components — safe to join
/// onto a trusted root directory without escaping it.
pub fn sanitize_relative_path(path: &str) -> PathBuf {
    Path::new(path)
        .clean()
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(PathBuf::from(part)),
            _ => None,
        })
        .collect()
}
