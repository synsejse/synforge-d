use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use synforge_core::{config::DaemonConfig, constants::DEFAULT_DAEMON_WORKER_SOCKET_PORT};

use crate::resources::WorkerResourceLimits;

pub(super) fn worker_container_body(
    config: &DaemonConfig,
    worker_id: &str,
    network_mode: Option<String>,
    binds: Option<Vec<String>>,
    resource_limits: WorkerResourceLimits,
) -> ContainerCreateBody {
    let cmd = vec![
        "--worker-id".to_string(),
        worker_id.to_string(),
        "--connect-addr".to_string(),
        format!("daemon:{}", DEFAULT_DAEMON_WORKER_SOCKET_PORT),
        "--socket-timeout-seconds".to_string(),
        config.worker_socket_timeout_seconds.to_string(),
    ];
    ContainerCreateBody {
        image: Some(config.worker_image.clone()),
        cmd: Some(cmd),
        host_config: Some(HostConfig {
            auto_remove: Some(true),
            privileged: Some(true),
            extra_hosts: Some(vec!["host.docker.internal:host-gateway".to_string()]),
            network_mode,
            binds,
            nano_cpus: resource_limits.nano_cpus,
            memory: resource_limits.memory_bytes,
            ..Default::default()
        }),
        working_dir: Some("/synforge".to_string()),
        ..Default::default()
    }
}

pub(super) async fn detect_daemon_network(docker: &Docker) -> Option<String> {
    let hostname = daemon_container_hostname().await?;
    let inspect = docker
        .inspect_container(
            &hostname,
            None::<bollard::query_parameters::InspectContainerOptions>,
        )
        .await
        .ok()?;
    let networks = inspect.network_settings?.networks?;
    networks
        .keys()
        .find(|name| {
            name.as_str() != "bridge" && name.as_str() != "host" && name.as_str() != "none"
        })
        .cloned()
        .or_else(|| networks.keys().next().cloned())
}

async fn daemon_container_hostname() -> Option<String> {
    tokio::fs::read_to_string("/etc/hostname")
        .await
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
