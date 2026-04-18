use synforge_core::api::ServerHardwareResponse;
use synforge_worker_host::HostHardwareService;

use super::SynforgeService;

impl SynforgeService {
    pub async fn get_server_hardware(&self) -> anyhow::Result<ServerHardwareResponse> {
        HostHardwareService.get_server_hardware().await
    }
}
