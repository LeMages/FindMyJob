-- Transforme status_history en timeline unifiée : chaque ligne devient un
-- événement typé (event_type), qu'il change le statut ou non (relance,
-- entretien planifié/passé, message, appel, note...). Ajoute aussi le statut
-- terminal 'ghosting' et le lien optionnel vers un entretien.
--
-- SQLite ne permet pas d'altérer un CHECK ni de rendre une colonne nullable
-- avec ALTER TABLE : candidatures et status_history sont recréées (même
-- technique que 0005_candidatures_add_interim.sql).

-- ----------------------------------------------------------------------
-- 1. candidatures : ajoute 'ghosting' au CHECK(status)
-- ----------------------------------------------------------------------

CREATE TABLE candidatures_v2 (
    id                     INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id                INTEGER NOT NULL REFERENCES users(id),
    poste                  TEXT NOT NULL,
    type                   TEXT CHECK(type IN ('stage', 'alternance', 'cdi', 'cdd', 'interim', 'freelance')),
    status                 TEXT NOT NULL DEFAULT 'a_postuler'
                           CHECK(status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon', 'ghosting')),
    source                 TEXT,
    url_offre              TEXT,
    description_offre      TEXT,
    entreprise             TEXT,
    localisation           TEXT,
    remuneration           TEXT,
    date_candidature       DATE,
    next_relance           DATE,
    last_relance           DATE,
    notes                  TEXT,
    created_at             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    session_id             INTEGER REFERENCES sessions(id),
    cv_path                TEXT,
    lettre_motivation_path TEXT
);

INSERT INTO candidatures_v2 (
    id, user_id, poste, type, status, source, url_offre, description_offre,
    entreprise, localisation, remuneration, date_candidature, next_relance,
    last_relance, notes, created_at, updated_at, session_id, cv_path,
    lettre_motivation_path
)
SELECT
    id, user_id, poste, type, status, source, url_offre, description_offre,
    entreprise, localisation, remuneration, date_candidature, next_relance,
    last_relance, notes, created_at, updated_at, session_id, cv_path,
    lettre_motivation_path
FROM candidatures;

DROP TABLE candidatures;
ALTER TABLE candidatures_v2 RENAME TO candidatures;

CREATE TRIGGER IF NOT EXISTS trg_candidatures_updated_at
AFTER UPDATE ON candidatures
FOR EACH ROW
BEGIN
    UPDATE candidatures SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- ----------------------------------------------------------------------
-- 2. status_history : ajoute event_type / interview_id, rend nouveau_status
--    nullable (un événement qui ne change pas le statut n'en écrit aucun),
--    et rétro-tague les lignes existantes.
-- ----------------------------------------------------------------------

CREATE TABLE status_history_v2 (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    candidature_id INTEGER NOT NULL REFERENCES candidatures(id),
    event_type     TEXT NOT NULL,
    ancien_status  TEXT CHECK(ancien_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon', 'ghosting')),
    nouveau_status TEXT
                   CHECK(nouveau_status IN ('a_postuler', 'postule', 'relance', 'entretien', 'offre', 'refus', 'abandon', 'ghosting')),
    content        TEXT,
    interview_id   INTEGER REFERENCES interviews(id),
    changed_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO status_history_v2 (
    id, candidature_id, event_type, ancien_status, nouveau_status, content, changed_at
)
SELECT
    id,
    candidature_id,
    CASE
        WHEN ancien_status IS NULL THEN 'creation'
        WHEN nouveau_status = 'abandon' THEN 'desistement'
        WHEN ancien_status = 'a_postuler' AND nouveau_status = 'postule' THEN 'candidature_envoyee'
        ELSE 'note'
    END,
    ancien_status,
    nouveau_status,
    content,
    changed_at
FROM status_history;

DROP TABLE status_history;
ALTER TABLE status_history_v2 RENAME TO status_history;

-- ----------------------------------------------------------------------
-- 3. interviews : lien optionnel vers le contact rencontré (dropdown du
--    formulaire "entretien planifié"). Aucun CHECK touché, ALTER simple.
-- ----------------------------------------------------------------------

ALTER TABLE interviews ADD COLUMN contact_id INTEGER REFERENCES contacts(id);
