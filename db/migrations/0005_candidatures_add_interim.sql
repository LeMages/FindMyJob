-- Add 'interim' to the allowed type values for candidatures.
-- SQLite does not support altering CHECK constraints, so the table is recreated.

CREATE TABLE candidatures_v2 (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id             INTEGER NOT NULL REFERENCES users(id),
    poste               TEXT NOT NULL,
    type                TEXT CHECK(type IN ('stage', 'alternance', 'cdi', 'cdd', 'interim', 'freelance')),
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

INSERT INTO candidatures_v2 SELECT * FROM candidatures;
DROP TABLE candidatures;
ALTER TABLE candidatures_v2 RENAME TO candidatures;

CREATE TRIGGER IF NOT EXISTS trg_candidatures_updated_at
AFTER UPDATE ON candidatures
FOR EACH ROW
BEGIN
    UPDATE candidatures SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
