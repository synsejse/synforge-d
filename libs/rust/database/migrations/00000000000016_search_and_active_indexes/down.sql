DROP INDEX IF EXISTS idx_build_jobs_active;
DROP INDEX IF EXISTS idx_packages_description_trgm;
DROP INDEX IF EXISTS idx_packages_name_trgm;
DROP INDEX IF EXISTS idx_build_jobs_mock_chroot_trgm;
DROP INDEX IF EXISTS idx_build_jobs_package_name_trgm;

-- Leave the pg_trgm extension installed: dropping it cascades onto
-- any other index that uses gin_trgm_ops, and the extension itself
-- is harmless if unused.
