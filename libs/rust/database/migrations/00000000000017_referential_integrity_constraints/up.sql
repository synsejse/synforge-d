-- Add foreign key constraints for child tables that have no meaning without
-- their parent row. Orphan rows (if any) are pruned first so the constraint
-- creation does not fail on existing data.

DELETE FROM published_repo_files
WHERE artifact_id NOT IN (SELECT id FROM build_artifacts);

DELETE FROM build_artifacts
WHERE job_id NOT IN (SELECT id FROM build_jobs);

DELETE FROM build_logs
WHERE job_id NOT IN (SELECT id FROM build_jobs);

DELETE FROM sync_operations
WHERE package_name NOT IN (SELECT name FROM packages);

DELETE FROM user_permissions
WHERE user_id NOT IN (SELECT id FROM users);

DELETE FROM user_repo_metrics
WHERE user_id NOT IN (SELECT id FROM users);

DELETE FROM build_failure_backoff
WHERE (package_name, mock_chroot) NOT IN (
    SELECT package_name, mock_chroot FROM package_mock_chroots
);

ALTER TABLE build_artifacts
    ADD CONSTRAINT fk_build_artifacts_job
    FOREIGN KEY (job_id) REFERENCES build_jobs (id)
    ON DELETE CASCADE;

ALTER TABLE build_logs
    ADD CONSTRAINT fk_build_logs_job
    FOREIGN KEY (job_id) REFERENCES build_jobs (id)
    ON DELETE CASCADE;

ALTER TABLE published_repo_files
    ADD CONSTRAINT fk_published_repo_files_artifact
    FOREIGN KEY (artifact_id) REFERENCES build_artifacts (id)
    ON DELETE CASCADE;

ALTER TABLE sync_operations
    ADD CONSTRAINT fk_sync_operations_package
    FOREIGN KEY (package_name) REFERENCES packages (name)
    ON DELETE CASCADE;

ALTER TABLE user_permissions
    ADD CONSTRAINT fk_user_permissions_user
    FOREIGN KEY (user_id) REFERENCES users (id)
    ON DELETE CASCADE;

ALTER TABLE user_repo_metrics
    ADD CONSTRAINT fk_user_repo_metrics_user
    FOREIGN KEY (user_id) REFERENCES users (id)
    ON DELETE CASCADE;

ALTER TABLE build_failure_backoff
    ADD CONSTRAINT fk_build_failure_backoff_target
    FOREIGN KEY (package_name, mock_chroot)
    REFERENCES package_mock_chroots (package_name, mock_chroot)
    ON DELETE CASCADE;
