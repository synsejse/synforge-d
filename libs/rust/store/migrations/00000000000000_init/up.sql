CREATE TABLE IF NOT EXISTS packages (
    name VARCHAR(255) PRIMARY KEY,
    description TEXT NOT NULL,
    enabled TINYINT(1) NOT NULL,
    repo_subdir TEXT NOT NULL,
    publish_srpm TINYINT(1) NOT NULL,
    mock_chroots_json TEXT NOT NULL,
    source_repo_url TEXT NOT NULL,
    source_spec_file TEXT NOT NULL,
    source_poll TINYINT(1) NOT NULL,
    poll_interval_seconds BIGINT NOT NULL,
    build_timeout_seconds BIGINT NOT NULL,
    package_history_count BIGINT NOT NULL DEFAULT 3,
    build_env_json TEXT NOT NULL,
    spec_file TEXT NOT NULL,
    version TEXT NOT NULL,
    `release` TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS build_jobs (
    id VARCHAR(36) PRIMARY KEY,
    package_name VARCHAR(255) NOT NULL,
    mock_chroot VARCHAR(255) NOT NULL,
    revision VARCHAR(255) NOT NULL,
    `trigger` VARCHAR(64) NOT NULL,
    status VARCHAR(64) NOT NULL,
    spec_file TEXT NOT NULL,
    worker_container_id VARCHAR(255),
    created_at VARCHAR(64) NOT NULL,
    updated_at VARCHAR(64) NOT NULL,
    finished_at VARCHAR(64),
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS build_artifacts (
    id VARCHAR(36) NOT NULL,
    job_id VARCHAR(36) NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    mock_chroot VARCHAR(255) NOT NULL,
    file VARCHAR(512) NOT NULL,
    sha256 VARCHAR(64) NOT NULL,
    size_bytes BIGINT NOT NULL,
    kind VARCHAR(64) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS build_logs (
    job_id VARCHAR(36) NOT NULL,
    file VARCHAR(255) NOT NULL,
    updated_at VARCHAR(64) NOT NULL,
    PRIMARY KEY (job_id, file)
);

CREATE TABLE IF NOT EXISTS published_repo_files (
    artifact_id VARCHAR(36) NOT NULL,
    published_at VARCHAR(64) NOT NULL,
    PRIMARY KEY (artifact_id)
);

CREATE INDEX idx_build_jobs_package_created_at
    ON build_jobs (package_name, created_at);
CREATE INDEX idx_build_jobs_package_status_created_at
    ON build_jobs (package_name, status, created_at);
CREATE INDEX idx_build_jobs_package_chroot_status_finished_at
    ON build_jobs (package_name, mock_chroot, status, finished_at);
CREATE INDEX idx_build_artifacts_job_id
    ON build_artifacts (job_id);
CREATE UNIQUE INDEX idx_build_artifacts_job_path
    ON build_artifacts (job_id, file);
