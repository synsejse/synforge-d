CREATE TABLE build_ccache_stats (
    job_id UUID PRIMARY KEY
        REFERENCES build_jobs (id) ON DELETE CASCADE,
    compiler_calls BIGINT NOT NULL CHECK (compiler_calls >= 0),
    direct_hits BIGINT NOT NULL CHECK (direct_hits >= 0),
    preprocessed_hits BIGINT NOT NULL CHECK (preprocessed_hits >= 0),
    cache_misses BIGINT NOT NULL CHECK (cache_misses >= 0),
    uncacheable_calls BIGINT NOT NULL CHECK (uncacheable_calls >= 0),
    error_calls BIGINT NOT NULL CHECK (error_calls >= 0),
    CHECK (
        compiler_calls = direct_hits
            + preprocessed_hits
            + cache_misses
            + uncacheable_calls
            + error_calls
    )
);
