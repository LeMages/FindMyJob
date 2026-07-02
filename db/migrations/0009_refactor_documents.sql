-- Refactor document storage: one PDF (artefact) per candidature instead of a
-- shared documents table. Reference CV/lettre de motivation are tracked on
-- disk only (fixed path, metadata read from the filesystem).

DROP TABLE IF EXISTS email_attachments;
DROP TABLE IF EXISTS documents;

ALTER TABLE candidatures DROP COLUMN cv_id;
ALTER TABLE candidatures ADD COLUMN cv_path TEXT;
ALTER TABLE candidatures ADD COLUMN lettre_motivation_path TEXT;
