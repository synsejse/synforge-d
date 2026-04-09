use bollard::Docker;
use synforge_core::api::ServerHardwareResponse;

use super::SynforgeService;

impl SynforgeService {
    pub async fn get_server_hardware(&self) -> anyhow::Result<ServerHardwareResponse> {
        let (cpu_cores, total_memory_mb) =
            if let Some((cpu_cores, total_memory_mb)) = read_docker_host_hardware().await {
                (cpu_cores, total_memory_mb)
            } else {
                (
                    read_total_cpu_cores().await.unwrap_or_else(|| {
                        std::thread::available_parallelism()
                            .map(|value| value.get() as u64)
                            .unwrap_or(1)
                    }),
                    read_total_memory_mb().await,
                )
            };
        Ok(ServerHardwareResponse {
            cpu_cores,
            total_memory_mb,
        })
    }
}

async fn read_docker_host_hardware() -> Option<(u64, Option<u64>)> {
    let docker = Docker::connect_with_local_defaults().ok()?;
    let info = docker.info().await.ok()?;
    let cpu_cores = info
        .ncpu
        .and_then(|value| u64::try_from(value).ok())
        .filter(|value| *value > 0)?;
    let total_memory_mb = info
        .mem_total
        .and_then(|value| u64::try_from(value).ok())
        .map(|value| value / (1024 * 1024))
        .filter(|value| *value > 0);
    Some((cpu_cores, total_memory_mb))
}

async fn read_total_cpu_cores() -> Option<u64> {
    let cpuinfo = tokio::fs::read_to_string("/proc/cpuinfo").await.ok()?;
    let count = cpuinfo
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count() as u64;
    (count > 0).then_some(count)
}

async fn read_total_memory_mb() -> Option<u64> {
    let meminfo = tokio::fs::read_to_string("/proc/meminfo").await.ok()?;
    let total_kb = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    Some(total_kb / 1024)
}
