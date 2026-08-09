diesel::table! {
    packages (name) {
        name -> Text,
        description -> Text,
        enabled -> Bool,
        repo_subdir -> Text,
        publish_srpm -> Bool,
        publish_debuginfo -> Bool,
        network_access -> Bool,
        source_repo_url -> Text,
        source_spec_file -> Text,
        source_poll -> Bool,
        poll_interval_seconds -> BigInt,
        build_timeout_seconds -> BigInt,
        package_history_count -> BigInt,
        cpu_limit_millicores -> Nullable<BigInt>,
        memory_limit_mb -> Nullable<BigInt>,
        ccache_enabled -> Bool,
        ccache_max_size_mb -> Nullable<BigInt>,
        build_env_json -> Jsonb,
        spec_file -> Text,
        version -> Text,
        release -> Text,
    }
}

diesel::table! {
    package_mock_chroots (package_name, mock_chroot) {
        package_name -> Text,
        mock_chroot -> Text,
    }
}

diesel::table! {
    build_jobs (id) {
        id -> Uuid,
        package_name -> Text,
        mock_chroot -> Text,
        revision -> Text,
        trigger -> Text,
        status -> Text,
        sync_operation_id -> Nullable<Uuid>,
        spec_file -> Text,
        worker_container_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
        signed_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    build_artifacts (id) {
        id -> Uuid,
        job_id -> Uuid,
        package_name -> Text,
        mock_chroot -> Text,
        file -> Text,
        sha256 -> Text,
        size_bytes -> BigInt,
        kind -> Text,
    }
}

diesel::table! {
    artifact_signatures (artifact_id) {
        artifact_id -> Uuid,
        status -> Text,
        signed_at -> Nullable<Timestamptz>,
        key_id -> Nullable<Text>,
        fingerprint -> Nullable<Text>,
        error_message -> Nullable<Text>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    build_logs (job_id, file) {
        job_id -> Uuid,
        file -> Text,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    build_failure_backoff (package_name, mock_chroot) {
        package_name -> Text,
        mock_chroot -> Text,
        consecutive_failures -> Integer,
        next_eligible_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        handle -> Text,
        display_name -> Text,
        password_hash -> Text,
        active -> Bool,
        is_bootstrap_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_permissions (user_id, permission) {
        user_id -> Uuid,
        permission -> Text,
    }
}

diesel::table! {
    user_repo_metrics (user_id) {
        user_id -> Uuid,
        downloaded_bytes -> BigInt,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    published_repo_files (artifact_id) {
        artifact_id -> Uuid,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    sync_batches (id) {
        id -> Uuid,
        trigger_type -> Text,
        status -> Text,
        total_packages -> BigInt,
        completed_packages -> BigInt,
        succeeded_packages -> BigInt,
        failed_packages -> BigInt,
        cancelled_packages -> BigInt,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    sync_operations (id) {
        id -> Uuid,
        package_name -> Text,
        trigger_type -> Text,
        status -> Text,
        stage -> Text,
        revision -> Nullable<Text>,
        previous_revision -> Nullable<Text>,
        changed -> Nullable<Bool>,
        target_mock_chroot -> Nullable<Text>,
        batch_id -> Nullable<Uuid>,
        retry_of -> Nullable<Uuid>,
        cancellation_requested -> Bool,
        queued_targets -> BigInt,
        skipped_targets -> BigInt,
        blocked_targets -> BigInt,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        finished_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    sync_operation_events (id) {
        id -> Uuid,
        sync_operation_id -> Uuid,
        stage -> Text,
        level -> Text,
        message -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    git_mirror_cache_states (mirror_key) {
        mirror_key -> Text,
        repo_url -> Text,
        last_fetched_at -> BigInt,
        last_used_at -> BigInt,
    }
}

diesel::table! {
    runtime_settings (key) {
        key -> Text,
        value_json -> Jsonb,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    packages,
    package_mock_chroots,
    build_jobs,
    build_artifacts,
    artifact_signatures,
    build_logs,
    build_failure_backoff,
    users,
    user_permissions,
    user_repo_metrics,
    published_repo_files,
    sync_batches,
    sync_operations,
    sync_operation_events,
    git_mirror_cache_states,
    runtime_settings,
);
