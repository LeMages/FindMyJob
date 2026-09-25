-- Retrait de `description_raw` (ajoutée en 0010) : seule la description
-- nettoyée (`description`) est conservée, le HTML source n'est pas stocké.
ALTER TABLE search.offers DROP COLUMN description_raw;
