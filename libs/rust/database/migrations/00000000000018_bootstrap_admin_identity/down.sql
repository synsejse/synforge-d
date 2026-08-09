DROP INDEX IF EXISTS users_single_bootstrap_admin_idx;

ALTER TABLE users
    DROP COLUMN IF EXISTS is_bootstrap_admin;
