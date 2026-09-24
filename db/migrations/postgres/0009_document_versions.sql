-- Historique des documents de référence (CV, lettre de motivation) d'une
-- session : chaque upload crée une version, jamais écrasée. Le dossier disque
-- d'une version est nommé par son `id` (jamais réutilisé) ; `version` n'est
-- qu'un numéro d'affichage (1, 2, 3…) par couple session/doc_type.
CREATE TABLE search.document_versions (
    id                BIGSERIAL   PRIMARY KEY,
    session_id        BIGINT      NOT NULL REFERENCES search.sessions(id) ON DELETE CASCADE,
    doc_type          TEXT        NOT NULL CHECK (doc_type IN ('cv', 'lettre_motivation')),
    version           INT         NOT NULL,
    original_filename TEXT        NOT NULL,
    typst_path        TEXT        NOT NULL,
    pdf_path          TEXT        NOT NULL,
    style_analysis    TEXT        NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_document_version UNIQUE (session_id, doc_type, version)
);

CREATE INDEX idx_document_versions_session ON search.document_versions(session_id, doc_type);

-- `documents` reste le pointeur vers la version courante (celle utilisée
-- par le pipeline de génération) ; NULL pour une référence antérieure au
-- versioning.
ALTER TABLE search.documents
    ADD COLUMN current_version_id BIGINT REFERENCES search.document_versions(id) ON DELETE SET NULL;

-- Version de référence ayant servi à générer les documents d'une
-- candidature ; NULL si générée avant le versioning.
ALTER TABLE search.candidatures
    ADD COLUMN cv_reference_version_id BIGINT REFERENCES search.document_versions(id) ON DELETE SET NULL,
    ADD COLUMN lm_reference_version_id BIGINT REFERENCES search.document_versions(id) ON DELETE SET NULL;
