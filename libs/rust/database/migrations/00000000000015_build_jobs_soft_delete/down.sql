DROP INDEX IF EXISTS idx_build_jobs_deleted_at;

ALTER TABLE build_jobs
    DROP COLUMN deleted_at;
