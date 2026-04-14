pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
pub const WORKER_JOBS_HOST_PATH_ENV_VAR: &str = "SYNFORGE_WORKER_JOBS_PATH";
pub const DEFAULT_LOG_ENV_FILTER: &str = "info,synforge=debug";

pub const DEFAULT_DAEMON_LISTEN_ADDR: &str = "0.0.0.0:8080";
pub const DEFAULT_DAEMON_PUBLIC_BASE_URL: &str = "http://localhost:8080";
pub const DEFAULT_DAEMON_WORKER_SOCKET_PORT: u16 = 8090;
pub const DEFAULT_DAEMON_WORKER_LISTEN_ADDR: &str = "0.0.0.0:8090";
pub const DEFAULT_SIGNING_PUBLIC_KEY_NAME: &str = "gpg.key";

pub const DAEMON_RUNTIME_ROOT: &str = "/var/lib/synforge";
pub const DAEMON_WORKER_RUNTIME_ROOT: &str = "/var/lib/synforge-worker-jobs";

pub const DEFAULT_WEBUI_STATIC_DIR: &str = "/opt/synforge/webui";
pub const DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_WORKER_SOCKET_TIMEOUT_SECONDS: u64 = 30;
pub const DEFAULT_WORKER_HEARTBEAT_INTERVAL_SECONDS: u64 = 10;
pub const DEFAULT_WORKER_WORKSPACE_ROOT: &str = "/tmp/synforge/jobs";
