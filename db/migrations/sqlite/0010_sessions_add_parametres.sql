ALTER TABLE sessions ADD COLUMN contexte TEXT;
ALTER TABLE sessions ADD COLUMN statut TEXT NOT NULL DEFAULT 'active' CHECK(statut IN ('active', 'pause', 'termine'));
ALTER TABLE sessions ADD COLUMN zone TEXT;
ALTER TABLE sessions ADD COLUMN cv_path TEXT;
ALTER TABLE sessions ADD COLUMN lm_path TEXT;
