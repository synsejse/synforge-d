use synforge_core::package::{BuildEnvVar, SpecRevision};

#[derive(Debug, Clone)]
pub struct InspectedPackageSource {
    pub package_name: String,
    pub description: String,
    pub revision: SpecRevision,
}

#[derive(Debug, Clone)]
pub struct PackageMaterializationOptions {
    pub enabled: bool,
    pub publish_srpm: bool,
    pub publish_debuginfo: bool,
    pub network_access: bool,
    pub ccache_enabled: bool,
    pub ccache_max_size_mb: Option<u64>,
    pub mock_chroots: Vec<String>,
    pub poll_interval_seconds: u64,
    pub build_timeout_seconds: u64,
    pub package_history_count: u64,
    pub cpu_limit_millicores: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub build_env: Vec<BuildEnvVar>,
}
