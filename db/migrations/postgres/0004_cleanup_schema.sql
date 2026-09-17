ALTER TABLE candidatures DROP COLUMN IF EXISTS last_relance;
ALTER TABLE candidatures DROP COLUMN IF EXISTS next_relance;

ALTER TABLE offers DROP COLUMN IF EXISTS embedding;

DROP TABLE IF EXISTS emails;
DROP TABLE IF EXISTS embeddings;
