-- Invert the CV/candidature relationship.
-- Before : documents.candidature_id -> candidatures  (a CV knows its candidature)
-- After  : candidatures.cv_id       -> documents     (a candidature knows its CV)

-- 1. Add cv_id to candidatures (nullable: existing rows get NULL)
ALTER TABLE candidatures ADD COLUMN cv_id INTEGER REFERENCES documents(id);

-- 2. Recreate documents without the candidature_id column
CREATE TABLE documents_v2 (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id            INTEGER NOT NULL REFERENCES users(id),
    source_document_id INTEGER REFERENCES documents_v2(id),
    type               TEXT NOT NULL CHECK(type IN ('cv', 'lettre_motivation')),
    label              TEXT,
    file_path          TEXT NOT NULL,
    is_reference       INTEGER NOT NULL DEFAULT 0 CHECK(is_reference IN (0, 1)),
    created_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO documents_v2 (id, user_id, source_document_id, type, label, file_path, is_reference, created_at, updated_at)
SELECT                    id, user_id, source_document_id, type, label, file_path, is_reference, created_at, updated_at
FROM documents;

DROP TABLE documents;
ALTER TABLE documents_v2 RENAME TO documents;

-- Recreate the trigger dropped with the original table
CREATE TRIGGER IF NOT EXISTS trg_documents_updated_at
AFTER UPDATE ON documents
FOR EACH ROW
BEGIN
    UPDATE documents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
