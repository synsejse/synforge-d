-- idx_build_jobs_package_created_at on (package_name, created_at) is a
-- strict prefix of idx_build_jobs_package_status_created_at on
-- (package_name, status, created_at). Postgres can use the wider index
-- for any query that the narrower one served, so the narrower one is
-- pure write-amplification and disk weight.
DROP INDEX IF EXISTS idx_build_jobs_package_created_at;
