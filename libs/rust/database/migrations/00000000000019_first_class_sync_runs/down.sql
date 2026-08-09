DROP INDEX IF EXISTS idx_build_jobs_sync_operation;

ALTER TABLE build_jobs
    DROP COLUMN IF EXISTS sync_operation_id;

DROP TABLE IF EXISTS sync_operation_events;

DROP INDEX IF EXISTS idx_sync_operations_batch_created;
DROP INDEX IF EXISTS uq_sync_operations_active_package;

ALTER TABLE sync_operations
    DROP COLUMN IF EXISTS finished_at,
    DROP COLUMN IF EXISTS started_at,
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS blocked_targets,
    DROP COLUMN IF EXISTS skipped_targets,
    DROP COLUMN IF EXISTS queued_targets,
    DROP COLUMN IF EXISTS cancellation_requested,
    DROP COLUMN IF EXISTS retry_of,
    DROP COLUMN IF EXISTS batch_id,
    DROP COLUMN IF EXISTS target_mock_chroot,
    DROP COLUMN IF EXISTS changed,
    DROP COLUMN IF EXISTS previous_revision,
    DROP COLUMN IF EXISTS stage;

DROP TABLE IF EXISTS sync_batches;
