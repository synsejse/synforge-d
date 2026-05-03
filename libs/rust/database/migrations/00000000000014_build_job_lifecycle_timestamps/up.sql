-- Track when a job actually started building and when its artifacts
-- finished signing, so the UI can distinguish "queued for N seconds"
-- from "running for N seconds" and surface signing latency separately.
ALTER TABLE build_jobs
    ADD COLUMN started_at TIMESTAMPTZ,
    ADD COLUMN signed_at  TIMESTAMPTZ;
