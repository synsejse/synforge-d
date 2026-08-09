CREATE TABLE sync_batches (
    id UUID PRIMARY KEY,
    trigger_type TEXT NOT NULL,
    status TEXT NOT NULL,
    total_packages BIGINT NOT NULL DEFAULT 0,
    completed_packages BIGINT NOT NULL DEFAULT 0,
    succeeded_packages BIGINT NOT NULL DEFAULT 0,
    failed_packages BIGINT NOT NULL DEFAULT 0,
    cancelled_packages BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    error_message TEXT
);

ALTER TABLE sync_operations
    ADD COLUMN stage TEXT NOT NULL DEFAULT 'completed',
    ADD COLUMN previous_revision TEXT,
    ADD COLUMN changed BOOLEAN,
    ADD COLUMN target_mock_chroot TEXT,
    ADD COLUMN batch_id UUID REFERENCES sync_batches (id) ON DELETE SET NULL,
    ADD COLUMN retry_of UUID REFERENCES sync_operations (id) ON DELETE SET NULL,
    ADD COLUMN cancellation_requested BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN queued_targets BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN skipped_targets BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN blocked_targets BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN updated_at TIMESTAMPTZ,
    ADD COLUMN started_at TIMESTAMPTZ,
    ADD COLUMN finished_at TIMESTAMPTZ;

UPDATE sync_operations
SET updated_at = created_at,
    started_at = created_at,
    finished_at = created_at;

ALTER TABLE sync_operations
    ALTER COLUMN updated_at SET NOT NULL;

CREATE UNIQUE INDEX uq_sync_operations_active_package
    ON sync_operations (package_name)
    WHERE status IN ('queued', 'running');

CREATE INDEX idx_sync_operations_batch_created
    ON sync_operations (batch_id, created_at)
    WHERE batch_id IS NOT NULL;

CREATE TABLE sync_operation_events (
    id UUID PRIMARY KEY,
    sync_operation_id UUID NOT NULL REFERENCES sync_operations (id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sync_operation_events_operation_created
    ON sync_operation_events (sync_operation_id, created_at);

ALTER TABLE build_jobs
    ADD COLUMN sync_operation_id UUID REFERENCES sync_operations (id) ON DELETE SET NULL;

CREATE INDEX idx_build_jobs_sync_operation
    ON build_jobs (sync_operation_id, created_at)
    WHERE sync_operation_id IS NOT NULL;
