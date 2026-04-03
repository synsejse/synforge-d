CREATE TABLE IF NOT EXISTS packages (
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    enabled BOOL NOT NULL,
    repo_subdir TEXT NOT NULL,
    publish_srpm BOOL NOT NULL,
    mock_chroots_json TEXT NOT NULL,
    source_repo_url TEXT NOT NULL,
    source_spec_path TEXT NOT NULL,
    source_poll BOOL NOT NULL,
    poll_interval_seconds BIGINT NOT NULL,
    build_timeout_seconds BIGINT NOT NULL,
    package_history_count BIGINT NOT NULL DEFAULT 3,
    build_env_json TEXT NOT NULL,
    spec_path TEXT NOT NULL,
    version TEXT NOT NULL,
    release TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS build_jobs (
    id TEXT PRIMARY KEY,
    package_name TEXT NOT NULL,
    mock_chroot TEXT NOT NULL,
    revision TEXT NOT NULL,
    trigger TEXT NOT NULL,
    status TEXT NOT NULL,
    spec_path TEXT NOT NULL,
    worker_container_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    finished_at TEXT,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS build_artifacts (
    job_id TEXT NOT NULL,
    package_name TEXT NOT NULL,
    mock_chroot TEXT NOT NULL,
    arch TEXT NOT NULL,
    path TEXT NOT NULL,
    relative_repo_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    kind TEXT NOT NULL,
    PRIMARY KEY (job_id, path)
);

CREATE TABLE IF NOT EXISTS build_logs (
    job_id TEXT PRIMARY KEY,
    log_path TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daemon_runtime_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    public_base_url TEXT
);

CREATE TABLE IF NOT EXISTS published_repo_files (
    job_id TEXT NOT NULL,
    package_name TEXT NOT NULL,
    mock_chroot TEXT NOT NULL,
    arch TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    kind TEXT NOT NULL,
    published_at TEXT NOT NULL,
    PRIMARY KEY (job_id, repo_path)
);

CREATE INDEX IF NOT EXISTS idx_build_jobs_package_created_at
    ON build_jobs (package_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_jobs_package_status_created_at
    ON build_jobs (package_name, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_jobs_package_chroot_status_finished_at
    ON build_jobs (package_name, mock_chroot, status, finished_at DESC);
CREATE INDEX IF NOT EXISTS idx_build_artifacts_job_id
    ON build_artifacts (job_id);
CREATE INDEX IF NOT EXISTS idx_published_repo_files_job_id
    ON published_repo_files (job_id);
CREATE INDEX IF NOT EXISTS idx_published_repo_files_package_published_at
    ON published_repo_files (package_name, published_at DESC);
