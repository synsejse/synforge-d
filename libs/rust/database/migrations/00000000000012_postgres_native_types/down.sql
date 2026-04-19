ALTER TABLE artifact_signatures
    DROP CONSTRAINT IF EXISTS fk_artifact_signatures_artifact;

ALTER TABLE build_jobs
    ALTER COLUMN id TYPE VARCHAR(36) USING id::text,
    ALTER COLUMN created_at TYPE VARCHAR(64) USING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    ALTER COLUMN finished_at TYPE VARCHAR(64) USING to_char(finished_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE build_artifacts
    ALTER COLUMN id TYPE VARCHAR(36) USING id::text,
    ALTER COLUMN job_id TYPE VARCHAR(36) USING job_id::text;

ALTER TABLE artifact_signatures
    ALTER COLUMN artifact_id TYPE VARCHAR(36) USING artifact_id::text,
    ALTER COLUMN signed_at TYPE VARCHAR(64) USING to_char(signed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE build_logs
    ALTER COLUMN job_id TYPE VARCHAR(36) USING job_id::text,
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE published_repo_files
    ALTER COLUMN artifact_id TYPE VARCHAR(36) USING artifact_id::text,
    ALTER COLUMN published_at TYPE VARCHAR(64) USING to_char(published_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE users
    ALTER COLUMN id TYPE VARCHAR(36) USING id::text,
    ALTER COLUMN created_at TYPE VARCHAR(64) USING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE user_permissions
    ALTER COLUMN user_id TYPE VARCHAR(36) USING user_id::text;

ALTER TABLE user_repo_metrics
    ALTER COLUMN user_id TYPE VARCHAR(36) USING user_id::text,
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE sync_operations
    ALTER COLUMN id TYPE VARCHAR(36) USING id::text,
    ALTER COLUMN created_at TYPE VARCHAR(64) USING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE runtime_settings
    ALTER COLUMN value_json TYPE TEXT USING value_json::text,
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE build_failure_backoff
    ALTER COLUMN next_eligible_at TYPE VARCHAR(64) USING to_char(next_eligible_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    ALTER COLUMN updated_at TYPE VARCHAR(64) USING to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"');

ALTER TABLE artifact_signatures
    ADD CONSTRAINT fk_artifact_signatures_artifact
        FOREIGN KEY (artifact_id) REFERENCES build_artifacts (id)
        ON DELETE CASCADE;
