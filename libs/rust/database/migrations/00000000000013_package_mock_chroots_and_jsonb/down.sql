ALTER TABLE packages
    ADD COLUMN mock_chroots_json TEXT NOT NULL DEFAULT '[]',
    ALTER COLUMN build_env_json TYPE TEXT USING build_env_json::text;

UPDATE packages
SET mock_chroots_json = COALESCE(
    (
        SELECT jsonb_agg(package_mock_chroots.mock_chroot ORDER BY package_mock_chroots.mock_chroot)::text
        FROM package_mock_chroots
        WHERE package_mock_chroots.package_name = packages.name
    ),
    '[]'
);

DROP INDEX IF EXISTS idx_package_mock_chroots_mock_chroot;
DROP TABLE IF EXISTS package_mock_chroots;
