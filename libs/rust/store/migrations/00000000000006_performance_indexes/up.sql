CREATE INDEX idx_build_jobs_status_created_at
    ON build_jobs (status, created_at);

CREATE INDEX idx_build_artifacts_package_target_kind_id
    ON build_artifacts (package_name, mock_chroot, kind, id);

CREATE INDEX idx_published_repo_files_published_at
    ON published_repo_files (published_at);

CREATE INDEX idx_sync_operations_package_status_created
    ON sync_operations (package_name, status, created_at);

CREATE INDEX idx_packages_enabled_name
    ON packages (enabled, name);

