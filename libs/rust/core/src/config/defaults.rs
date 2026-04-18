use crate::constants::{DEFAULT_DAEMON_LISTEN_ADDR, DEFAULT_DAEMON_PUBLIC_BASE_URL};

pub(super) fn default_listen_addr() -> String {
    DEFAULT_DAEMON_LISTEN_ADDR.to_string()
}

pub(super) fn default_max_concurrent_builds() -> usize {
    2
}

pub(super) fn default_db_pool_size() -> u32 {
    5
}

pub(super) fn default_queue_buffer_size() -> usize {
    128
}

pub(super) fn default_poller_tick_seconds() -> u64 {
    30
}

pub(super) fn default_worker_result_timeout_seconds() -> u64 {
    10
}

pub(super) fn default_worker_socket_timeout_seconds() -> u64 {
    30
}

pub(super) fn default_git_operation_timeout_seconds() -> u64 {
    600
}

pub(super) fn default_public_base_url() -> String {
    DEFAULT_DAEMON_PUBLIC_BASE_URL.to_string()
}

pub(super) fn default_redis_key_prefix() -> String {
    "synforge".to_string()
}

pub(super) fn default_mock_chroot_cache_ttl_seconds() -> u64 {
    300
}

pub(super) fn default_git_mirror_refresh_ttl_seconds() -> u64 {
    300
}

pub(super) fn default_git_mirror_max_unused_seconds() -> u64 {
    604_800
}

pub(super) fn default_build_failure_backoff_base_seconds() -> u64 {
    300
}

pub(super) fn default_build_failure_backoff_max_seconds() -> u64 {
    21_600
}

pub(super) fn default_signing_enabled() -> bool {
    false
}

pub(super) fn default_bootstrap_completed() -> bool {
    false
}

pub(super) fn default_worker_image() -> String {
    "synforge-worker-fedora:latest".to_string()
}
