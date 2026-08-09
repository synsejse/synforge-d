-- A package/mock target may have at most one pending or running job.
--
-- Older releases used a read-then-insert check, so clean up any historical
-- duplicates before installing the database-enforced invariant. Keep the
-- newest active row and fail older duplicates with an explicit reason.
WITH ranked_active_jobs AS (
    SELECT
        id,
        ROW_NUMBER() OVER (
            PARTITION BY package_name, mock_chroot
            ORDER BY created_at DESC, id DESC
        ) AS active_rank
    FROM build_jobs
    WHERE status IN ('pending', 'running') AND deleted_at IS NULL
)
UPDATE build_jobs
SET
    status = 'failed',
    updated_at = NOW(),
    finished_at = COALESCE(finished_at, NOW()),
    error_message = 'superseded while enforcing one active job per target'
WHERE id IN (
    SELECT id
    FROM ranked_active_jobs
    WHERE active_rank > 1
);

CREATE UNIQUE INDEX uq_build_jobs_active_target
    ON build_jobs (package_name, mock_chroot)
    WHERE status IN ('pending', 'running') AND deleted_at IS NULL;
