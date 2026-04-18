CREATE TABLE IF NOT EXISTS build_failure_backoff (
    package_name VARCHAR(255) NOT NULL,
    mock_chroot VARCHAR(255) NOT NULL,
    consecutive_failures INT NOT NULL,
    next_eligible_at VARCHAR(64) NOT NULL,
    updated_at VARCHAR(64) NOT NULL,
    PRIMARY KEY (package_name, mock_chroot)
);

CREATE INDEX idx_build_failure_backoff_next_eligible
    ON build_failure_backoff (next_eligible_at);
