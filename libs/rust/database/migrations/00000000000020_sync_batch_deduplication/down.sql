ALTER TABLE sync_batches
    DROP COLUMN IF EXISTS enqueue_failed_packages,
    DROP COLUMN IF EXISTS deduplicated_packages;
