diesel::table! {
    packages (name) {
        name -> Text,
        description -> Text,
        enabled -> Bool,
        repo_subdir -> Text,
        publish_srpm -> Bool,
        network_access -> Bool,
        mock_chroots_json -> Text,
        source_repo_url -> Text,
        source_spec_path -> Text,
        source_poll -> Bool,
        poll_interval_seconds -> BigInt,
        build_timeout_seconds -> BigInt,
        package_history_count -> BigInt,
        build_env_json -> Text,
        spec_path -> Text,
        version -> Text,
        release -> Text,
    }
}

diesel::table! {
    build_jobs (id) {
        id -> Text,
        package_name -> Text,
        mock_chroot -> Text,
        revision -> Text,
        trigger -> Text,
        status -> Text,
        spec_path -> Text,
        worker_container_id -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        finished_at -> Nullable<Text>,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    build_artifacts (id) {
        id -> Text,
        job_id -> Text,
        package_name -> Text,
        mock_chroot -> Text,
        path -> Text,
        sha256 -> Text,
        size_bytes -> BigInt,
        kind -> Text,
    }
}

diesel::table! {
    build_logs (job_id, source_path) {
        job_id -> Text,
        source_path -> Text,
        log_path -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        handle -> Text,
        display_name -> Text,
        password_hash -> Text,
        active -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    user_permissions (user_id, permission) {
        user_id -> Text,
        permission -> Text,
    }
}

diesel::table! {
    user_repo_metrics (user_id) {
        user_id -> Text,
        downloaded_bytes -> BigInt,
        updated_at -> Text,
    }
}

diesel::table! {
    published_repo_files (artifact_id) {
        artifact_id -> Text,
        repo_path -> Text,
        published_at -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    packages,
    build_jobs,
    build_artifacts,
    build_logs,
    users,
    user_permissions,
    user_repo_metrics,
    published_repo_files,
);
