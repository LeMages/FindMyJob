-- FindMyJob : carnet de contacts global (migration 0004).
-- Un contact existe indépendamment des candidatures et peut être lié à
-- plusieurs (`candidature_contacts`) ; un entretien peut avoir plusieurs
-- interlocuteurs (`interview_contacts`). `contacts.candidature_id` et
-- `interviews.contact_id` disparaissent, `contacts.nom` devient obligatoire.
-- Reprise : texte trimé (vide → NULL), un contact sans nom prend son email
-- comme nom, un contact sans nom ni email est supprimé (et ses liens avec).
--
-- Le migrateur exécute ce fichier dans une transaction, clés étrangères
-- actives (PRAGMA foreign_keys = OFF y est sans effet). Les DROP TABLE
-- déclenchent donc les actions des clés étrangères : les liens sont copiés
-- dans des tables de travail avant les DROP, et `status_history.interview_id`
-- (remis à NULL par le DROP de `interviews`) est restauré ensuite.

-- ------------------------------------------------------------------
-- 1. Nouveau carnet, contacts retenus
-- ------------------------------------------------------------------
CREATE TABLE contacts_new (
                              id         INTEGER PRIMARY KEY,
                              nom        TEXT NOT NULL,
                              role       TEXT,
                              email      TEXT,
                              telephone  TEXT,
                              notes      TEXT,
                              created_at TEXT NOT NULL
) STRICT;

INSERT INTO contacts_new (id, nom, role, email, telephone, notes, created_at)
SELECT id,
       COALESCE(NULLIF(TRIM(nom), ''), NULLIF(TRIM(email), '')),
       NULLIF(TRIM(role), ''),
       NULLIF(TRIM(email), ''),
       NULLIF(TRIM(telephone), ''),
       NULLIF(TRIM(notes), ''),
       created_at
  FROM contacts
 WHERE COALESCE(NULLIF(TRIM(nom), ''), NULLIF(TRIM(email), '')) IS NOT NULL;

-- ------------------------------------------------------------------
-- 2. Tables de travail : liens à reprendre et historique d'entretien
-- ------------------------------------------------------------------
CREATE TABLE migration_candidature_contacts AS
SELECT candidature_id, id AS contact_id, created_at
  FROM contacts
 WHERE id IN (SELECT id FROM contacts_new);

CREATE TABLE migration_interview_contacts AS
SELECT id AS interview_id, contact_id
  FROM interviews
 WHERE contact_id IN (SELECT id FROM contacts_new);

CREATE TABLE migration_status_history_interviews AS
SELECT id, interview_id
  FROM status_history
 WHERE interview_id IS NOT NULL;

-- ------------------------------------------------------------------
-- 3. Entretiens sans contact_id
-- ------------------------------------------------------------------
CREATE TABLE interviews_new (
                                id             INTEGER PRIMARY KEY,
                                candidature_id INTEGER NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
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

INSERT INTO interviews_new
       (id, candidature_id, type_entretien, modalite, date_entretien, lieu, issue, notes, created_at)
SELECT id, candidature_id, type_entretien, modalite, date_entretien, lieu, issue, notes, created_at
  FROM interviews;

-- ------------------------------------------------------------------
-- 4. Remplacement des tables
-- ------------------------------------------------------------------
-- Le DROP de `interviews` (avant `contacts`, qu'il référence) remet
-- `status_history.interview_id` à NULL : restauré à l'étape 5.
DROP TABLE interviews;
DROP TABLE contacts;
ALTER TABLE contacts_new RENAME TO contacts;
ALTER TABLE interviews_new RENAME TO interviews;
CREATE INDEX idx_interviews_candidature ON interviews(candidature_id);

-- ------------------------------------------------------------------
-- 5. Restauration de l'entretien des lignes d'historique
-- ------------------------------------------------------------------
UPDATE status_history
   SET interview_id = (SELECT m.interview_id
                         FROM migration_status_history_interviews m
                        WHERE m.id = status_history.id)
 WHERE id IN (SELECT id FROM migration_status_history_interviews);

-- ------------------------------------------------------------------
-- 6. Tables de liaison et reprise des liens
-- ------------------------------------------------------------------
CREATE TABLE candidature_contacts (
                                      candidature_id INTEGER NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
                                      contact_id     INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
                                      created_at     TEXT NOT NULL,
                                      PRIMARY KEY (candidature_id, contact_id)
) STRICT;
CREATE INDEX idx_candidature_contacts_contact ON candidature_contacts(contact_id);

CREATE TABLE interview_contacts (
                                    interview_id INTEGER NOT NULL REFERENCES interviews(id) ON DELETE CASCADE,
                                    contact_id   INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
                                    PRIMARY KEY (interview_id, contact_id)
) STRICT;

INSERT INTO candidature_contacts (candidature_id, contact_id, created_at)
SELECT candidature_id, contact_id, created_at
  FROM migration_candidature_contacts;

INSERT INTO interview_contacts (interview_id, contact_id)
SELECT interview_id, contact_id
  FROM migration_interview_contacts;

DROP TABLE migration_candidature_contacts;
DROP TABLE migration_interview_contacts;
DROP TABLE migration_status_history_interviews;
