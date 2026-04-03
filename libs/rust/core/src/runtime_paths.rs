use std::path::{Path, PathBuf};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    metadata_root: PathBuf,
    database_path: PathBuf,
    packages_dir: PathBuf,
    repo_dir: PathBuf,
    jobs_root: PathBuf,
}

impl RuntimePaths {
    pub fn new(
        database_path: PathBuf,
        packages_dir: PathBuf,
        repo_dir: PathBuf,
        jobs_root: PathBuf,
    ) -> Self {
        let metadata_root = database_path
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("/var/lib/synforge/metadata"));

        Self {
            metadata_root,
            database_path,
            packages_dir,
            repo_dir,
            jobs_root,
        }
    }

    pub fn metadata_root(&self) -> &Path {
        &self.metadata_root
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn packages_dir(&self) -> &Path {
        &self.packages_dir
    }

    pub fn repo_dir(&self) -> &Path {
        &self.repo_dir
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

    pub fn temp_root(&self) -> PathBuf {
        self.metadata_root.join("tmp")
    }

    pub fn parse_workspace_dir(&self, job_id: Uuid) -> PathBuf {
        self.temp_root().join("parse").join(job_id.to_string())
    }

    pub fn browse_workspace_dir(&self, browse_id: Uuid) -> PathBuf {
        self.temp_root().join("browse").join(browse_id.to_string())
    }

    pub fn package_repo_dir(&self, package_name: &str, release: &str, arch: &str) -> PathBuf {
        self.repo_dir
            .join("packages")
            .join(package_name)
            .join(release)
            .join(arch)
    }
}
