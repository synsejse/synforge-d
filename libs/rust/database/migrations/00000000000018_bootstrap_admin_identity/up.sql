ALTER TABLE users
    ADD COLUMN is_bootstrap_admin BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE users
SET is_bootstrap_admin = TRUE
WHERE id = (
    SELECT id
    FROM users
    ORDER BY created_at ASC, id ASC
    LIMIT 1
);

CREATE UNIQUE INDEX users_single_bootstrap_admin_idx
    ON users (is_bootstrap_admin)
    WHERE is_bootstrap_admin;

INSERT INTO runtime_settings (key, value_json, updated_at)
SELECT 'bootstrap_completed', 'true'::jsonb, CURRENT_TIMESTAMP
WHERE EXISTS (SELECT 1 FROM users)
ON CONFLICT (key) DO NOTHING;
