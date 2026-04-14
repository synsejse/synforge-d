CREATE TABLE sync_operations (
    id VARCHAR(36) PRIMARY KEY,
    package_name VARCHAR(255) NOT NULL,
    trigger_type VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    revision VARCHAR(255),
    error_message TEXT,
    created_at VARCHAR(64) NOT NULL,
    INDEX idx_package_created (package_name, created_at),
    INDEX idx_status_created (status, created_at)
);