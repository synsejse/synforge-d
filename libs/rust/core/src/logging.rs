use tracing_subscriber::EnvFilter;

const DEFAULT_ENV_FILTER: &str = "info,synforge=debug";

pub fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| DEFAULT_ENV_FILTER.into());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_names(true)
        .init();
}
