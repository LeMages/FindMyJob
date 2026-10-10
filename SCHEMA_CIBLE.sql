-- FindMyJob : schéma cible SQLite, figé le 2026-10-10.
-- Conventions :
--   - dates-heures : texte ISO 8601 UTC, fournies par le code Rust ;
--   - dates simples : texte 'YYYY-MM-DD' ;
--   - booléens : 0 ou 1 ;
--   - chemins de fichiers : relatifs au dossier de données, séparateur '/' ;
--   - PRAGMA foreign_keys = ON à chaque connexion ;
--   - aucun trigger : updated_at est écrit par le code ;
--   - AUTOINCREMENT sur les tables dont l'id nomme un dossier (jamais réutilisé).

-- ============================ Profil ============================

CREATE TABLE users (
                       id          INTEGER PRIMARY KEY CHECK (id = 1),   -- app mono-utilisateur
                       nom         TEXT NOT NULL,
                       prenom      TEXT NOT NULL,
                       email       TEXT NOT NULL,
                       telephone   TEXT,
                       adresse     TEXT,
                       code_postal TEXT,
                       ville       TEXT,
                       permis      INTEGER CHECK (permis IN (0, 1)),   -- NULL = non renseigné
                       description TEXT,
                       linkedin    TEXT,
                       github      TEXT,
                       portfolio   TEXT,
                       interets    TEXT,
                       created_at  TEXT NOT NULL
) STRICT;

CREATE TABLE experiences (
                             id           INTEGER PRIMARY KEY,
                             poste        TEXT NOT NULL,
                             entreprise   TEXT,
                             lieu         TEXT,
                             description  TEXT,
                             date_debut   TEXT,
                             date_fin     TEXT,
                             type_contrat TEXT CHECK (type_contrat IN
                                                      ('stage','alternance','cdi','cdd','interim','freelance')),
                             created_at   TEXT NOT NULL
) STRICT;

CREATE TABLE formations (
                            id          INTEGER PRIMARY KEY,
                            titre       TEXT NOT NULL,
                            ecole       TEXT,
                            description TEXT,
                            date_debut  TEXT,
                            date_fin    TEXT,
                            created_at  TEXT NOT NULL
) STRICT;

CREATE TABLE projets (
                         id          INTEGER PRIMARY KEY,
                         titre       TEXT NOT NULL,
                         description TEXT,
                         stack       TEXT,
                         url         TEXT,
                         pour_qui    TEXT,
                         date_debut  TEXT,
                         date_fin    TEXT,
                         created_at  TEXT NOT NULL
) STRICT;

CREATE TABLE skills (
                        id         INTEGER PRIMARY KEY,
                        domaine    TEXT NOT NULL,
                        nom        TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        UNIQUE (domaine, nom)
) STRICT;

CREATE TABLE langues (
                         id         INTEGER PRIMARY KEY,
                         nom        TEXT NOT NULL UNIQUE,
                         level      TEXT,
                         created_at TEXT NOT NULL
) STRICT;

CREATE TABLE settings (
                          cle    TEXT PRIMARY KEY,
                          valeur TEXT
) STRICT;

-- ===================== Sessions et documents =====================

CREATE TABLE sessions (
                          id         INTEGER PRIMARY KEY AUTOINCREMENT,
                          nom        TEXT NOT NULL,
                          statut     TEXT NOT NULL DEFAULT 'active'
                              CHECK (statut IN ('active','pause','termine')),
                          created_at TEXT NOT NULL
) STRICT;

CREATE TABLE document_versions (
                                   id                INTEGER PRIMARY KEY AUTOINCREMENT,
                                   session_id        INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                                   doc_type          TEXT NOT NULL CHECK (doc_type IN ('cv','lettre_motivation')),
                                   version           INTEGER NOT NULL,
                                   original_filename TEXT NOT NULL,
                                   typst_path        TEXT NOT NULL,
                                   pdf_path          TEXT NOT NULL,
                                   template_path     TEXT,
                                   example_data_path TEXT,
                                   page_count        INTEGER,
                                   created_at        TEXT NOT NULL,
                                   UNIQUE (session_id, doc_type, version)
) STRICT;

-- Pointeur vers la version courante de chaque référence.
CREATE TABLE documents (
                           session_id         INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                           doc_type           TEXT NOT NULL CHECK (doc_type IN ('cv','lettre_motivation')),
                           current_version_id INTEGER NOT NULL
                               REFERENCES document_versions(id) ON DELETE CASCADE,
                           updated_at         TEXT NOT NULL,
                           PRIMARY KEY (session_id, doc_type)
) STRICT;

-- ========================= Candidatures =========================

CREATE TABLE candidatures (
                              id                      INTEGER PRIMARY KEY AUTOINCREMENT,
                              session_id              INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                              poste                   TEXT NOT NULL,
                              type_contrat            TEXT NOT NULL CHECK (type_contrat IN
                                                                           ('stage','alternance','cdi','cdd','interim','freelance')),
                              status                  TEXT NOT NULL DEFAULT 'a_postuler' CHECK (status IN
                                                                                                ('a_postuler','postule','relance','entretien',
                                                                                                 'entretien_passe','offre','refus','abandon',
                                                                                                 'ghosting','accepte','offre_refusee')),
                              source                  TEXT NOT NULL,
                              url_offre               TEXT,
                              description_offre       TEXT NOT NULL,
                              entreprise              TEXT NOT NULL,
                              localisation            TEXT NOT NULL,
                              remuneration            TEXT,
                              notes                   TEXT,
                              cv_path                 TEXT,
                              lettre_motivation_path  TEXT,
                              cv_reference_version_id INTEGER REFERENCES document_versions(id) ON DELETE SET NULL,
                              lm_reference_version_id INTEGER REFERENCES document_versions(id) ON DELETE SET NULL,
                              created_at              TEXT NOT NULL
) STRICT;

CREATE TABLE contacts (
                          id             INTEGER PRIMARY KEY,
                          candidature_id INTEGER NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
                          nom            TEXT,
                          role           TEXT,
                          email          TEXT,
                          telephone      TEXT,
                          notes          TEXT,
                          created_at     TEXT NOT NULL
) STRICT;

CREATE TABLE interviews (
                            id             INTEGER PRIMARY KEY,
                            candidature_id INTEGER NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
                            contact_id     INTEGER REFERENCES contacts(id) ON DELETE SET NULL,
                            type_entretien TEXT CHECK (type_entretien IN
                                                       ('rh','technique','manager','final','autre')),
                            modalite       TEXT CHECK (modalite IN ('visio','presentiel','telephone')),
                            date_entretien TEXT,
                            lieu           TEXT,
                            issue          TEXT NOT NULL DEFAULT 'en_attente'
                                CHECK (issue IN ('en_attente','positif','negatif')),
                            notes          TEXT,
                            created_at     TEXT NOT NULL
) STRICT;

CREATE TABLE status_history (
                                id             INTEGER PRIMARY KEY,
                                candidature_id INTEGER NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
                                event_type     TEXT NOT NULL CHECK (event_type IN
                                                ('creation','candidature_envoyee','relance','entretien_planifie',
                                                 'entretien_passe','offre_recue','refus_recu','ghosting',
                                                 'desistement','accepter_offre','decliner_offre','message_envoye',
                                                 'message_recu','appel','note')),
                                ancien_status  TEXT CHECK (ancien_status IN ('a_postuler','postule','relance','entretien',
                                                        'entretien_passe','offre','refus','abandon',
                                                        'ghosting','accepte','offre_refusee')),
                                nouveau_status TEXT CHECK (nouveau_status IN ('a_postuler','postule','relance','entretien',
                                                        'entretien_passe','offre','refus','abandon',
                                                        'ghosting','accepte','offre_refusee')),
                                content        TEXT,
                                interview_id   INTEGER REFERENCES interviews(id) ON DELETE SET NULL,
                                changed_at     TEXT NOT NULL
) STRICT;

CREATE INDEX idx_candidatures_session     ON candidatures(session_id);
CREATE INDEX idx_contacts_candidature     ON contacts(candidature_id);
CREATE INDEX idx_interviews_candidature   ON interviews(candidature_id);
CREATE INDEX idx_status_history_candidature ON status_history(candidature_id);