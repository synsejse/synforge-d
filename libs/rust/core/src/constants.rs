pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
pub const DEFAULT_LOG_ENV_FILTER: &str = "info,synforge=debug";

pub const DEFAULT_DAEMON_LISTEN_ADDR: &str = "0.0.0.0:8080";
pub const DEFAULT_DAEMON_HTTP_PORT: u16 = 8080;
pub const DEFAULT_DAEMON_PUBLIC_BASE_URL: &str = "http://localhost:8080";
pub const DEFAULT_DAEMON_WORKER_SOCKET_PORT: u16 = 8090;
pub const DEFAULT_DAEMON_WORKER_LISTEN_ADDR: &str = "0.0.0.0:8090";
pub const DEFAULT_SIGNING_PUBLIC_KEY_NAME: &str = "gpg.key";

pub const DEFAULT_WEBUI_LISTEN_ADDR: &str = "0.0.0.0:80";
pub const DEFAULT_WEBUI_STATIC_DIR: &str = "/opt/synforge/webui";
pub const DEFAULT_WEBUI_MAX_REQUEST_BODY_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_WORKER_SOCKET_TIMEOUT_SECONDS: u64 = 30;
pub const DEFAULT_WORKER_HEARTBEAT_INTERVAL_SECONDS: u64 = 10;
