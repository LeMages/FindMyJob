-- FindMyJob — Initial schema
-- Idempotent: CREATE TABLE/TRIGGER/INDEX IF NOT EXISTS

CREATE TABLE IF NOT EXISTS users (
                                     id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                     nom         TEXT,
                                     prenom      TEXT,
                                     email       TEXT,
                                     telephone   TEXT,
                                     ville       TEXT,
                                     permis      INTEGER CHECK(permis IN (0, 1)),
    description TEXT,
    linkedin    TEXT,
    github      TEXT,
    portfolio   TEXT,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS lang (
                                    id      INTEGER PRIMARY KEY AUTOINCREMENT,
                                    user_id INTEGER NOT NULL REFERENCES users(id),
    nom     TEXT,
    level   TEXT
    );

CREATE TABLE IF NOT EXISTS projets (
                                       id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                       user_id     INTEGER NOT NULL REFERENCES users(id),
    titre       TEXT NOT NULL,
    description TEXT,
    stack       TEXT,
    url         TEXT,
    date_debut  DATE,
    date_fin    DATE,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS experiences (
                                           id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                           user_id     INTEGER NOT NULL REFERENCES users(id),
    poste       TEXT NOT NULL,
    entreprise  TEXT,
    lieu        TEXT,
    description TEXT,
    date_debut  DATE,
    date_fin    DATE,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS skills (
                                      id         INTEGER PRIMARY KEY AUTOINCREMENT,
                                      user_id    INTEGER NOT NULL REFERENCES users(id),
    domaine    TEXT NOT NULL,
    nom        TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS candidatures (
                                            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                                            user_id             INTEGER NOT NULL REFERENCES users(id),
    poste               TEXT NOT NULL,
    type                TEXT CHECK(type IN ('stage', 'alternance', 'cdi', 'cdd', 'freelance')),
    status              TEXT NOT NULL DEFAULT 'a_postuler'
    CHECK(status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon')),
    source              TEXT,
    url_offre           TEXT,
    description_offre   TEXT,
    entreprise          TEXT,
    localisation        TEXT,
    remuneration        TEXT,
    date_candidature    DATE,
    next_relance        DATE,
    last_relance        DATE,
    notes               TEXT,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS status_history (
                                              id               INTEGER PRIMARY KEY AUTOINCREMENT,
                                              candidature_id   INTEGER NOT NULL REFERENCES candidatures(id),
    ancien_status    TEXT CHECK(ancien_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon')),
    nouveau_status   TEXT NOT NULL
    CHECK(nouveau_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon')),
    changed_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS contacts (
                                        id             INTEGER PRIMARY KEY AUTOINCREMENT,
                                        candidature_id INTEGER NOT NULL REFERENCES candidatures(id),
    nom            TEXT,
    role           TEXT,
    email          TEXT,
    telephone      TEXT,
    notes          TEXT,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS interviews (
                                          id              INTEGER PRIMARY KEY AUTOINCREMENT,
                                          candidature_id  INTEGER NOT NULL REFERENCES candidatures(id),
    type            TEXT CHECK(type IN ('rh', 'technique', 'manager', 'final', 'autre')),
    date_entretien  TIMESTAMP,
    lieu            TEXT,
    issue           TEXT DEFAULT 'en_attente'
    CHECK(issue IN ('en_attente', 'positif', 'negatif')),
    notes           TEXT,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS documents (
                                         id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                                         user_id            INTEGER NOT NULL REFERENCES users(id),
    source_document_id INTEGER REFERENCES documents(id),
    candidature_id     INTEGER REFERENCES candidatures(id),
    type               TEXT NOT NULL CHECK(type IN ('cv', 'lettre_motivation')),
    label              TEXT,
    file_path          TEXT NOT NULL,
    is_reference       INTEGER NOT NULL DEFAULT 0 CHECK(is_reference IN (0, 1)),
    created_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS emails (
                                      id             INTEGER PRIMARY KEY AUTOINCREMENT,
                                      candidature_id INTEGER NOT NULL REFERENCES candidatures(id),
    contact_id     INTEGER REFERENCES contacts(id),
    type           TEXT CHECK(type IN ('candidature', 'relance', 'remerciement', 'reponse', 'autre')),
    sujet          TEXT,
    corps          TEXT,
    destinataire   TEXT,
    sent_at        TIMESTAMP,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE TABLE IF NOT EXISTS email_attachments (
                                                 id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                                 email_id    INTEGER NOT NULL REFERENCES emails(id),
    document_id INTEGER NOT NULL REFERENCES documents(id)
    );

CREATE TABLE IF NOT EXISTS embeddings (
                                          id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                          entity_type TEXT NOT NULL,
                                          entity_id   INTEGER NOT NULL,
                                          model       TEXT NOT NULL,
                                          dimensions  INTEGER,
                                          vector      BLOB NOT NULL,
                                          created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_embeddings_entity ON embeddings(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS settings (
                                        id         INTEGER PRIMARY KEY AUTOINCREMENT,
                                        cle        TEXT NOT NULL UNIQUE,
                                        valeur     TEXT,
                                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- updated_at triggers

CREATE TRIGGER IF NOT EXISTS trg_users_updated_at
AFTER UPDATE ON users
                            FOR EACH ROW
BEGIN
UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_projets_updated_at
AFTER UPDATE ON projets
                            FOR EACH ROW
BEGIN
UPDATE projets SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_experiences_updated_at
AFTER UPDATE ON experiences
                            FOR EACH ROW
BEGIN
UPDATE experiences SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_skills_updated_at
AFTER UPDATE ON skills
                            FOR EACH ROW
BEGIN
UPDATE skills SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_candidatures_updated_at
AFTER UPDATE ON candidatures
                            FOR EACH ROW
BEGIN
UPDATE candidatures SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_contacts_updated_at
AFTER UPDATE ON contacts
                            FOR EACH ROW
BEGIN
UPDATE contacts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_interviews_updated_at
AFTER UPDATE ON interviews
                            FOR EACH ROW
BEGIN
UPDATE interviews SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_documents_updated_at
AFTER UPDATE ON documents
                            FOR EACH ROW
BEGIN
UPDATE documents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_settings_updated_at
AFTER UPDATE ON settings
                            FOR EACH ROW
BEGIN
UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;