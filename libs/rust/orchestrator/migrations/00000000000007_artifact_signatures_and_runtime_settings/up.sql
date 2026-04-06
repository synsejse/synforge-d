CREATE TABLE IF NOT EXISTS artifact_signatures (
    artifact_id VARCHAR(36) NOT NULL,
    status VARCHAR(32) NOT NULL,
    signed_at VARCHAR(64),
    key_id TEXT,
    fingerprint TEXT,
    error_message TEXT,
    updated_at VARCHAR(64) NOT NULL,
    PRIMARY KEY (artifact_id),
    CONSTRAINT fk_artifact_signatures_artifact
        FOREIGN KEY (artifact_id) REFERENCES build_artifacts (id)
        ON DELETE CASCADE
);

CREATE INDEX idx_artifact_signatures_status_updated
    ON artifact_signatures (status, updated_at);

CREATE TABLE IF NOT EXISTS runtime_settings (
    `key` VARCHAR(191) NOT NULL,
    value_json TEXT NOT NULL,
    updated_at VARCHAR(64) NOT NULL,
    PRIMARY KEY (`key`)
);
