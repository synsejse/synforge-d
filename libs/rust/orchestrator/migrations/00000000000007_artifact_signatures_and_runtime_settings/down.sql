DROP TABLE IF EXISTS runtime_settings;

DROP INDEX idx_artifact_signatures_status_updated ON artifact_signatures;
DROP TABLE IF EXISTS artifact_signatures;
