use super::DockerWorkerLauncher;
use synforge_core::{
    config::DaemonConfig,
    model::{WorkerAction, WorkerJobPayload},
};

impl DockerWorkerLauncher {
    pub(super) async fn mock_mount_binds(
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let WorkerAction::Build(build) = &payload.action else {
            return Ok(None);
        };

        let host_jobs_root = config.worker_jobs_host_path().ok_or_else(|| {
            anyhow::anyhow!("SYNFORGE_WORKER_JOBS_PATH is required for build worker bind mounts")
        })?;
        let host_ccache_root = config.worker_ccache_host_path().ok_or_else(|| {
            anyhow::anyhow!("SYNFORGE_WORKER_JOBS_PATH is required for worker ccache bind mounts")
        })?;
        let host_mock_cache_root = config.worker_mock_cache_host_path().ok_or_else(|| {
            anyhow::anyhow!(
                "SYNFORGE_WORKER_JOBS_PATH is required for worker mock cache bind mounts"
            )
        })?;

        let host_mock_lib = host_jobs_root
            .join(payload.job_id.to_string())
            .join("mock")
            .join("lib");
        let host_mock_cache_dir = host_mock_cache_root
            .join(&build.package.name)
            .join(&build.mock_chroot);

        let container_jobs_root = config.worker_jobs_root();
        let container_mock_root = container_jobs_root
            .join(payload.job_id.to_string())
            .join("mock");
        let container_mock_lib_dir = container_mock_root.join("lib");
        let container_mock_cache_dir = config
            .worker_mock_cache_root()
            .join(&build.package.name)
            .join(&build.mock_chroot);
        tokio::fs::create_dir_all(&container_mock_lib_dir).await?;
        tokio::fs::create_dir_all(&container_mock_cache_dir).await?;

        let worker_mock_root = payload.workspace_dir.join("mock");
        let worker_mock_lib = worker_mock_root.join("lib");
        let worker_mock_cache = worker_mock_root.join("cache");

        let mut binds = vec![
            format!(
                "{}:{}:rw,z",
                host_mock_lib.display(),
                worker_mock_lib.display()
            ),
            format!(
                "{}:{}:rw,z",
                host_mock_cache_dir.display(),
                worker_mock_cache.display()
            ),
        ];

        if build.package.ccache_enabled {
            let host_ccache_dir = host_ccache_root
                .join(&build.package.name)
                .join(&build.mock_chroot);
            let container_ccache_dir = config
                .worker_ccache_root()
                .join(&build.package.name)
                .join(&build.mock_chroot);
            tokio::fs::create_dir_all(&container_ccache_dir).await?;
            let worker_ccache_dir = payload.workspace_dir.join("ccache");
            binds.push(format!(
                "{}:{}:rw,z",
                host_ccache_dir.display(),
                worker_ccache_dir.display()
            ));
        }

        Ok(Some(binds))
    }
}
