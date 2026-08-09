ALTER TABLE sync_batches
    ADD COLUMN deduplicated_packages BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN enqueue_failed_packages BIGINT NOT NULL DEFAULT 0;
