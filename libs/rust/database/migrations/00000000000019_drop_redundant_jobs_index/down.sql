CREATE INDEX IF NOT EXISTS idx_build_jobs_package_created_at
    ON build_jobs (package_name, created_at);
