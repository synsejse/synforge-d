CREATE TABLE package_mock_chroots (
    package_name VARCHAR(255) NOT NULL,
    mock_chroot VARCHAR(255) NOT NULL,
    PRIMARY KEY (package_name, mock_chroot),
    CONSTRAINT fk_package_mock_chroots_package
        FOREIGN KEY (package_name) REFERENCES packages (name)
        ON DELETE CASCADE
);

INSERT INTO package_mock_chroots (package_name, mock_chroot)
SELECT packages.name, value
FROM packages,
LATERAL jsonb_array_elements_text(packages.mock_chroots_json::jsonb) AS value;

CREATE INDEX idx_package_mock_chroots_mock_chroot
    ON package_mock_chroots (mock_chroot, package_name);

ALTER TABLE packages
    ALTER COLUMN build_env_json TYPE JSONB USING build_env_json::jsonb,
    DROP COLUMN mock_chroots_json;
