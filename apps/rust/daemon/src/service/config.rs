use std::collections::BTreeMap;

use serde_json::Value;
use synforge_core::{
    api::{
        ConfigSchemaResponse, EffectiveConfigDto, EffectiveConfigView, SetupInitializeRequest,
        UpdateRuntimeSettingsRequest,
    },
    config::{
        DaemonConfig, RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED, apply_config_settings,
        apply_live_config_settings, daemon_config_runtime_settings, editable_config_fields,
        ensure_runtime_editable_keys, ensure_setup_editable_keys, pending_restart_config_settings,
    },
    validation::{validate_display_name, validate_password, validate_user_handle},
};
use synforge_database::DieselStore;
use synforge_database::runtime_settings::PostgresRuntimeSettingsStore;
use synforge_database::users::PostgresUserStore;
use synforge_publish::RepoSigningManager;

use super::SynforgeService;

pub(crate) async fn load_effective_daemon_config_from_store(
    base_config: &DaemonConfig,
    store: &DieselStore,
) -> anyhow::Result<DaemonConfig> {
    let mut current = base_config.clone();
    let runtime_settings = PostgresRuntimeSettingsStore::new(store.clone());
    let dynamic_settings = runtime_settings.list().await?;
    apply_live_config_settings(&mut current, &dynamic_settings)?;
    let mut updates = BTreeMap::new();
    if !current.bootstrap_completed && PostgresUserStore::new(store.clone()).user_count().await? > 0
    {
        current.bootstrap_completed = true;
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    } else if current.bootstrap_completed && !dynamic_settings.contains_key("bootstrap_completed") {
        updates.insert("bootstrap_completed".to_string(), Value::Bool(true));
    }
    if !updates.is_empty() {
        runtime_settings.upsert(updates).await?;
    }
    current
        .validate()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(current)
}

impl SynforgeService {
    pub async fn config_schema(&self) -> ConfigSchemaResponse {
        ConfigSchemaResponse {
            fields: editable_config_fields(),
        }
    }

    pub(crate) async fn load_effective_daemon_config(&self) -> anyhow::Result<DaemonConfig> {
        load_effective_daemon_config_from_store(&self.config, &self.store).await
    }

    pub async fn effective_config(&self) -> anyhow::Result<EffectiveConfigDto> {
        let current = self.load_effective_daemon_config().await?;
        let stored_settings = PostgresRuntimeSettingsStore::new(self.store.clone())
            .list()
            .await?;
        let pending_restart_settings = pending_restart_config_settings(&current, &stored_settings);
        let paths = current.runtime_paths();
        Ok(EffectiveConfigDto {
            config: EffectiveConfigView {
                bootstrap_completed: current.bootstrap_completed,
                listen_addr: current.listen_addr.clone(),
                database_url: current.database_url.clone(),
                redis_url: current.redis_url.clone(),
                redis_key_prefix: current.redis_key_prefix.clone(),
                repo_dir: paths.repo_dir().to_path_buf(),
                jobs_root: paths.jobs_root().to_path_buf(),
                cache_root: paths.cache_root().to_path_buf(),
                work_root: paths.work_root().to_path_buf(),
                signing_root: paths.signing_root(),
                worker_jobs_root: current.worker_jobs_root(),
                worker_ccache_root: current.worker_ccache_root(),
                worker_image: current.worker_image.clone(),
                signing_enabled: current.signing_enabled,
                signing_key_id: current.signing_key_id.clone(),
                max_concurrent_builds: current.max_concurrent_builds,
                db_pool_size: current.db_pool_size,
                queue_buffer_size: current.queue_buffer_size,
                poller_tick_seconds: current.poller_tick_seconds,
                worker_result_timeout_seconds: current.worker_result_timeout_seconds,
                worker_socket_timeout_seconds: current.worker_socket_timeout_seconds,
                git_operation_timeout_seconds: current.git_operation_timeout_seconds,
                public_base_url: current.public_base_url,
                mock_chroot_cache_ttl_seconds: current.mock_chroot_cache_ttl_seconds,
                git_mirror_refresh_ttl_seconds: current.git_mirror_refresh_ttl_seconds,
                git_mirror_max_unused_seconds: current.git_mirror_max_unused_seconds,
                build_failure_backoff_base_seconds: current.build_failure_backoff_base_seconds,
                build_failure_backoff_max_seconds: current.build_failure_backoff_max_seconds,
            },
            restart_required: !pending_restart_settings.is_empty(),
            pending_restart_settings,
        })
    }

    pub async fn update_runtime_settings(
        &self,
        request: UpdateRuntimeSettingsRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let request_settings = request.settings;
        ensure_runtime_editable_keys(&request_settings)?;

        let mut config = self.load_effective_daemon_config().await?;
        let runtime_settings = PostgresRuntimeSettingsStore::new(self.store.clone());
        let mut merged_settings = runtime_settings.list().await?;
        for (key, value) in request_settings.clone() {
            merged_settings.insert(key, value);
        }
        apply_config_settings(&mut config, &merged_settings, true)?;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;

        runtime_settings.upsert(request_settings).await?;
        self.effective_config().await
    }

    pub async fn initialize_setup(
        &self,
        request: SetupInitializeRequest,
    ) -> anyhow::Result<EffectiveConfigDto> {
        let SetupInitializeRequest {
            settings,
            admin,
            enable_signing,
            signing_armored_private_key,
        } = request;
        let current = self.load_effective_daemon_config().await?;
        if current.bootstrap_completed {
            anyhow::bail!("setup has already been completed");
        }
        validate_user_handle(&admin.handle)?;
        validate_display_name(&admin.display_name)?;
        validate_password(&admin.password)?;
        let request_settings = settings;
        ensure_setup_editable_keys(&request_settings)?;
        let mut config = current;
        apply_config_settings(&mut config, &request_settings, false)?;
        config.bootstrap_completed = false;
        config
            .validate()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if PostgresUserStore::new(self.store.clone())
            .user_count()
            .await?
            > 0
        {
            anyhow::bail!("initial admin cannot be created because users already exist");
        }
        self.bootstrap_admin(&admin.handle, &admin.display_name, &admin.password)
            .await?;
        let mut runtime_settings = daemon_config_runtime_settings(&config);
        let signing_manager = RepoSigningManager;
        if enable_signing {
            let managed_key = if let Some(armored_private_key) = signing_armored_private_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                signing_manager.remove_all_keys(&config).await?;
                signing_manager
                    .import_private_key(&config, armored_private_key)
                    .await?
            } else {
                signing_manager.ensure_managed_signing_key(&config).await?
            };
            let armored_private_key = signing_manager
                .export_private_key(&config, managed_key.key_id.as_str())
                .await?;
            runtime_settings.insert(
                "signing_key_id".to_string(),
                Value::String(managed_key.key_id.clone()),
            );
            runtime_settings.insert("signing_enabled".to_string(), Value::Bool(true));
            runtime_settings.insert(
                RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
                Value::String(armored_private_key),
            );
            config.signing_enabled = true;
            config.signing_key_id = Some(managed_key.key_id);
        } else {
            runtime_settings.insert("signing_enabled".to_string(), Value::Bool(false));
            runtime_settings.insert("signing_key_id".to_string(), Value::Null);
            runtime_settings.insert(
                RUNTIME_SETTING_SIGNING_PRIVATE_KEY_ARMORED.to_string(),
                Value::Null,
            );
            config.signing_enabled = false;
            config.signing_key_id = None;
        }
        config.bootstrap_completed = true;
        runtime_settings.insert("bootstrap_completed".to_string(), Value::Bool(true));
        PostgresRuntimeSettingsStore::new(self.store.clone())
            .upsert(runtime_settings)
            .await?;
        signing_manager
            .reconcile_repo_metadata_signature(&config, config.runtime_paths().repo_dir())
            .await?;
        self.effective_config().await
    }
}
