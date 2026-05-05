ALTER TABLE build_failure_backoff DROP CONSTRAINT IF EXISTS fk_build_failure_backoff_target;
ALTER TABLE user_repo_metrics DROP CONSTRAINT IF EXISTS fk_user_repo_metrics_user;
ALTER TABLE user_permissions DROP CONSTRAINT IF EXISTS fk_user_permissions_user;
ALTER TABLE sync_operations DROP CONSTRAINT IF EXISTS fk_sync_operations_package;
ALTER TABLE published_repo_files DROP CONSTRAINT IF EXISTS fk_published_repo_files_artifact;
ALTER TABLE build_logs DROP CONSTRAINT IF EXISTS fk_build_logs_job;
ALTER TABLE build_artifacts DROP CONSTRAINT IF EXISTS fk_build_artifacts_job;
