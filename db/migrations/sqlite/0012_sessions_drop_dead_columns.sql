-- Nettoyage des colonnes mortes : sessions.cv_path / sessions.lm_path sont
-- remplacées par la table documents (0011). DROP COLUMN nécessite SQLite
-- >= 3.35.0 (le projet embarque SQLite 3.46 via libsqlite3-sys, cf. Cargo.lock).

ALTER TABLE sessions DROP COLUMN cv_path;
ALTER TABLE sessions DROP COLUMN lm_path;
