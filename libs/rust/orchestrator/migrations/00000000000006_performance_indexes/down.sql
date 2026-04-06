DROP INDEX idx_packages_enabled_name ON packages;

DROP INDEX idx_sync_operations_package_status_created ON sync_operations;

DROP INDEX idx_published_repo_files_published_at ON published_repo_files;

DROP INDEX idx_build_artifacts_package_target_kind_id ON build_artifacts;

DROP INDEX idx_build_jobs_status_created_at ON build_jobs;

