CREATE TABLE git_mirror_cache_states (
    mirror_key VARCHAR(64) PRIMARY KEY,
    repo_url TEXT NOT NULL,
    last_fetched_at BIGINT NOT NULL,
    last_used_at BIGINT NOT NULL,
    INDEX idx_git_mirror_cache_last_used (last_used_at)
);
