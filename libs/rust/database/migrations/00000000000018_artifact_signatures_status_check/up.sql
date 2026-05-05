-- Restrict artifact_signatures.status to the valid values used by the
-- ArtifactSigningStatus enum in synforge-core.

ALTER TABLE artifact_signatures
    ADD CONSTRAINT chk_artifact_signatures_status
    CHECK (status IN ('signed', 'failed', 'skipped'));
