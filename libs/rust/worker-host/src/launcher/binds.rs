use synforge_core::{
    config::DaemonConfig,
    model::{WorkerAction, WorkerJobPayload},
};
use tracing::warn;

use super::DockerWorkerLauncher;

impl DockerWorkerLauncher {
    pub(super) async fn mock_mount_binds(
        payload: &WorkerJobPayload,
        config: &DaemonConfig,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let WorkerAction::Build(build) = &payload.action else {
            return Ok(None);
        };

        let Some(host_jobs_root) = config.worker_jobs_host_path() else {
            warn!(
                job_id = %payload.job_id,
                package_name = %build.package.name,
                mock_chroot = %build.mock_chroot,
                ccache_enabled = build.package.ccache_enabled,
                "SYNFORGE_WORKER_JOBS_PATH not set; workers will run without dedicated mock or ccache bind mounts"
            );
            return Ok(None);
        };
        let host_ccache_root = config.worker_ccache_host_path();

        let host_mock_root = host_jobs_root.join(payload.job_id.to_string()).join("mock");
        let host_mock_lib = host_mock_root.join("lib");
        let host_mock_cache = host_mock_root.join("cache");

        let container_jobs_root = config.worker_jobs_root();
        let container_mock_root = container_jobs_root
            .join(payload.job_id.to_string())
            .join("mock");
        let container_mock_lib_dir = container_mock_root.join("lib");
        let container_mock_cache_dir = container_mock_root.join("cache");
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
                host_mock_cache.display(),
                worker_mock_cache.display()
            ),
        ];

        if build.package.ccache_enabled {
            let Some(host_ccache_root) = host_ccache_root else {
                warn!(
                    job_id = %payload.job_id,
                    package_name = %build.package.name,
                    mock_chroot = %build.mock_chroot,
                    "worker ccache requested but host worker cache root is unavailable"
                );
                return Ok(Some(binds));
            };
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
