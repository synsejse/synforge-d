ALTER TABLE artifact_signatures
    DROP CONSTRAINT IF EXISTS fk_artifact_signatures_artifact;

ALTER TABLE build_jobs
    ALTER COLUMN id TYPE UUID USING id::uuid,
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at::timestamptz,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz,
    ALTER COLUMN finished_at TYPE TIMESTAMPTZ USING finished_at::timestamptz;

ALTER TABLE build_artifacts
    ALTER COLUMN id TYPE UUID USING id::uuid,
    ALTER COLUMN job_id TYPE UUID USING job_id::uuid;

ALTER TABLE artifact_signatures
    ALTER COLUMN artifact_id TYPE UUID USING artifact_id::uuid,
    ALTER COLUMN signed_at TYPE TIMESTAMPTZ USING signed_at::timestamptz,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE build_logs
    ALTER COLUMN job_id TYPE UUID USING job_id::uuid,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE published_repo_files
    ALTER COLUMN artifact_id TYPE UUID USING artifact_id::uuid,
    ALTER COLUMN published_at TYPE TIMESTAMPTZ USING published_at::timestamptz;

ALTER TABLE users
    ALTER COLUMN id TYPE UUID USING id::uuid,
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at::timestamptz,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE user_permissions
    ALTER COLUMN user_id TYPE UUID USING user_id::uuid;

ALTER TABLE user_repo_metrics
    ALTER COLUMN user_id TYPE UUID USING user_id::uuid,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE sync_operations
    ALTER COLUMN id TYPE UUID USING id::uuid,
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at::timestamptz;

ALTER TABLE runtime_settings
    ALTER COLUMN value_json TYPE JSONB USING value_json::jsonb,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE build_failure_backoff
    ALTER COLUMN next_eligible_at TYPE TIMESTAMPTZ USING next_eligible_at::timestamptz,
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at::timestamptz;

ALTER TABLE artifact_signatures
    ADD CONSTRAINT fk_artifact_signatures_artifact
        FOREIGN KEY (artifact_id) REFERENCES build_artifacts (id)
        ON DELETE CASCADE;
