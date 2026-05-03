-- Two indexes that pull weight on hot paths.
--
-- 1. `pg_trgm` GIN indexes on the columns we substring-search.
--    `list_jobs` does `package_name LIKE '%foo%'` and
--    `mock_chroot LIKE '%bar%'` (jobs list filters); `list_packages`
--    does the same on `name` and `description`. Without trigram
--    indexes Postgres has no choice but to seq-scan. Trigram GIN
--    turns LIKE/ILIKE '%pattern%' queries into index lookups.
--
-- 2. Partial index on the active-jobs hot path. Pending+running rows
--    are tiny at any moment, but we hit them constantly:
--    `has_active_job_for_target`, `list_active_jobs`, the dashboard
--    pipeline strip, and the package-runtime DISTINCT ON for
--    "active job per target". A partial index keyed on
--    (package_name, mock_chroot, created_at DESC) covers all four
--    shapes and stays small because of the WHERE clause.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX idx_build_jobs_package_name_trgm
    ON build_jobs USING gin (package_name gin_trgm_ops);

CREATE INDEX idx_build_jobs_mock_chroot_trgm
    ON build_jobs USING gin (mock_chroot gin_trgm_ops);

CREATE INDEX idx_packages_name_trgm
    ON packages USING gin (name gin_trgm_ops);

CREATE INDEX idx_packages_description_trgm
    ON packages USING gin (description gin_trgm_ops);

CREATE INDEX idx_build_jobs_active
    ON build_jobs (package_name, mock_chroot, created_at DESC)
    WHERE status IN ('pending', 'running') AND deleted_at IS NULL;
