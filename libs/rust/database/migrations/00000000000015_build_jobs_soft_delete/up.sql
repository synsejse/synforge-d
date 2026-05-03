-- Soft-delete for build jobs: pruning a job (manual delete, prune-failed,
-- or auto-prune of old successful builds) drops the heavy data
-- (artifacts, logs, signatures, published-file rows, on-disk dir) but
-- keeps the build_jobs row so historical statistics still see it.
ALTER TABLE build_jobs
    ADD COLUMN deleted_at TIMESTAMPTZ;

-- Partial index for the few queries that want only soft-deleted rows
-- (audit / future purge). Active queries filter `deleted_at IS NULL` and
-- are already served by the existing (package_name, status, ...) indexes.
CREATE INDEX idx_build_jobs_deleted_at
    ON build_jobs (deleted_at)
    WHERE deleted_at IS NOT NULL;
