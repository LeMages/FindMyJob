-- Table formations (diplômes, écoles)
CREATE TABLE IF NOT EXISTS formations (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER NOT NULL REFERENCES users(id),
  titre       TEXT NOT NULL,
  ecole       TEXT,
  date_debut  DATE,
  date_fin    DATE,
  created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER IF NOT EXISTS trg_formations_updated_at
AFTER UPDATE ON formations
FOR EACH ROW
BEGIN
  UPDATE formations SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
