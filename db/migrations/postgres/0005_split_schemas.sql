-- Scinde la base en deux schémas Postgres : `profile` (identité utilisateur,
-- CV, compétences, settings) et `search` (sessions de recherche, offres,
-- candidatures, entretiens, documents). App mono-utilisateur : les tables
-- du schéma search ne portent plus de user_id.
SET search_path TO public, profile, search;

CREATE SCHEMA IF NOT EXISTS profile;
CREATE SCHEMA IF NOT EXISTS search;

-- ------------------------------------------------------------------
-- Déplacement des tables vers profile
-- ------------------------------------------------------------------
ALTER TABLE users SET SCHEMA profile;
ALTER TABLE lang SET SCHEMA profile;
ALTER TABLE formations SET SCHEMA profile;
ALTER TABLE projets SET SCHEMA profile;
ALTER TABLE experiences SET SCHEMA profile;
ALTER TABLE skills SET SCHEMA profile;
ALTER TABLE settings SET SCHEMA profile;

-- ------------------------------------------------------------------
-- Déplacement des tables vers search
-- ------------------------------------------------------------------
ALTER TABLE sessions SET SCHEMA search;
ALTER TABLE offers SET SCHEMA search;
ALTER TABLE candidatures SET SCHEMA search;
ALTER TABLE status_history SET SCHEMA search;
ALTER TABLE contacts SET SCHEMA search;
ALTER TABLE interviews SET SCHEMA search;
ALTER TABLE documents SET SCHEMA search;

-- ------------------------------------------------------------------
-- user_id devenu inutile (app mono-utilisateur)
-- ------------------------------------------------------------------
ALTER TABLE search.sessions DROP COLUMN user_id;
ALTER TABLE search.candidatures DROP COLUMN user_id;

-- ------------------------------------------------------------------
-- candidatures.session_id devient obligatoire
-- ------------------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM search.candidatures WHERE session_id IS NULL) THEN
        RAISE EXCEPTION 'candidatures contient des lignes avec session_id NULL, à rattacher avant migration';
    END IF;
END $$;

ALTER TABLE search.candidatures ALTER COLUMN session_id SET NOT NULL;

-- ------------------------------------------------------------------
-- offers rattachées à une session
-- ------------------------------------------------------------------
ALTER TABLE search.offers ADD COLUMN session_id BIGINT;

-- Offres scrapées existantes sans session : données régénérables, on les
-- supprime plutôt que d'inventer un rattachement arbitraire.
DELETE FROM search.offers WHERE session_id IS NULL;

ALTER TABLE search.offers ALTER COLUMN session_id SET NOT NULL;

ALTER TABLE search.offers
    ADD CONSTRAINT fk_offers_session
    FOREIGN KEY (session_id) REFERENCES search.sessions(id);

CREATE INDEX idx_offers_session_id ON search.offers(session_id);

-- ------------------------------------------------------------------
-- search_path par défaut de la base : le code applicatif continue à
-- référencer les tables sans préfixe de schéma.
-- ------------------------------------------------------------------
ALTER DATABASE findmyjob SET search_path TO public, profile, search;
