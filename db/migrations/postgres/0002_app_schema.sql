CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    nom             TEXT,
    prenom          TEXT,
    email           TEXT,
    telephone       TEXT,
    ville           TEXT,
    permis          BOOLEAN      DEFAULT FALSE,
    description     TEXT,
    linkedin        TEXT,
    github          TEXT,
    portfolio       TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE lang (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    nom             TEXT,
    level           TEXT
);

CREATE TABLE projets (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    titre           TEXT         NOT NULL,
    description     TEXT,
    stack           TEXT,
    url             TEXT,
    date_debut      DATE,
    date_fin        DATE,
    pour_qui        TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE experiences (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    poste           TEXT         NOT NULL,
    entreprise      TEXT,
    lieu            TEXT,
    description     TEXT,
    date_debut      DATE,
    date_fin        DATE,
    type_contrat    TEXT         CHECK (type_contrat IN ('stage', 'alternance', 'interim', 'cdd', 'cdi')),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE formations (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    titre           TEXT         NOT NULL,
    ecole           TEXT,
    date_debut      DATE,
    date_fin        DATE,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE skills (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    domaine         TEXT         NOT NULL,
    nom             TEXT         NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE sessions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT       NOT NULL REFERENCES users(id),
    nom             TEXT         NOT NULL,
    contexte        TEXT,
    statut          TEXT         NOT NULL DEFAULT 'active' CHECK (statut IN ('active', 'pause', 'termine')),
    zone            TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE candidatures (
    id                      BIGSERIAL PRIMARY KEY,
    user_id                 BIGINT       NOT NULL REFERENCES users(id),
    session_id              BIGINT       REFERENCES sessions(id),
    poste                   TEXT         NOT NULL,
    type                    TEXT         CHECK (type IN ('stage', 'alternance', 'cdi', 'cdd', 'interim', 'freelance')),
    status                  TEXT         NOT NULL DEFAULT 'a_postuler' CHECK (status IN ('a_postuler', 'postule', 'relance', 'entretien', 'entretien_passe', 'offre', 'refus', 'abandon', 'ghosting', 'accepte', 'offre_refusee')),
    source                  TEXT,
    url_offre               TEXT,
    description_offre       TEXT,
    entreprise              TEXT,
    localisation            TEXT,
    remuneration            TEXT,
    date_candidature        DATE,
    next_relance            DATE,
    last_relance            DATE,
    notes                   TEXT,
    cv_path                 TEXT,
    lettre_motivation_path  TEXT,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE contacts (
    id              BIGSERIAL PRIMARY KEY,
    candidature_id  BIGINT       NOT NULL REFERENCES candidatures(id),
    nom             TEXT,
    role            TEXT,
    email           TEXT,
    telephone       TEXT,
    notes           TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE interviews (
    id              BIGSERIAL PRIMARY KEY,
    candidature_id  BIGINT       NOT NULL REFERENCES candidatures(id),
    contact_id      BIGINT       REFERENCES contacts(id),
    type            TEXT         CHECK (type IN ('rh', 'technique', 'manager', 'final', 'autre')),
    modalite        TEXT         CHECK (modalite IN ('visio', 'presentiel', 'telephone')),
    date_entretien  TIMESTAMPTZ,
    lieu            TEXT,
    issue           TEXT         DEFAULT 'en_attente' CHECK (issue IN ('en_attente', 'positif', 'negatif')),
    notes           TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE status_history (
    id              BIGSERIAL PRIMARY KEY,
    candidature_id  BIGINT       NOT NULL REFERENCES candidatures(id),
    event_type      TEXT         NOT NULL,
    ancien_status   TEXT         CHECK (ancien_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'entretien_passe', 'offre', 'refus', 'abandon', 'ghosting', 'accepte', 'offre_refusee')),
    nouveau_status  TEXT         CHECK (nouveau_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'entretien_passe', 'offre', 'refus', 'abandon', 'ghosting', 'accepte', 'offre_refusee')),
    content         TEXT,
    interview_id    BIGINT       REFERENCES interviews(id),
    changed_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE documents (
    id                  BIGSERIAL PRIMARY KEY,
    session_id          BIGINT       NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    doc_type            TEXT         NOT NULL CHECK (doc_type IN ('cv', 'lettre_motivation')),
    original_filename   TEXT         NOT NULL,
    file_path           TEXT         NOT NULL,
    pdf_path            TEXT,
    style_analysis      TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_documents_session_doctype UNIQUE (session_id, doc_type)
);

CREATE TABLE emails (
    id              BIGSERIAL PRIMARY KEY,
    candidature_id  BIGINT       NOT NULL REFERENCES candidatures(id),
    contact_id      BIGINT       REFERENCES contacts(id),
    type            TEXT         CHECK (type IN ('candidature', 'relance', 'remerciement', 'reponse', 'autre')),
    sujet           TEXT,
    corps           TEXT,
    destinataire    TEXT,
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE embeddings (
    id              BIGSERIAL PRIMARY KEY,
    entity_type     TEXT         NOT NULL,
    entity_id       BIGINT       NOT NULL,
    model           TEXT         NOT NULL,
    dimensions      INTEGER,
    vector          vector(1536) NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_embeddings_entity ON embeddings (entity_type, entity_id);

CREATE TABLE settings (
    id              BIGSERIAL PRIMARY KEY,
    cle             TEXT         NOT NULL,
    valeur          TEXT,
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_settings_cle UNIQUE (cle)
);

-- Fonction réutilisable pour maintenir updated_at à jour automatiquement.
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_projets_updated_at
    BEFORE UPDATE ON projets
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_experiences_updated_at
    BEFORE UPDATE ON experiences
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_formations_updated_at
    BEFORE UPDATE ON formations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_skills_updated_at
    BEFORE UPDATE ON skills
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_candidatures_updated_at
    BEFORE UPDATE ON candidatures
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_contacts_updated_at
    BEFORE UPDATE ON contacts
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_interviews_updated_at
    BEFORE UPDATE ON interviews
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER trg_settings_updated_at
    BEFORE UPDATE ON settings
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
