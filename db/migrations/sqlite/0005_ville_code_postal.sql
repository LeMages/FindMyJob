-- FindMyJob : le lieu du profil devient `ville` + `zip_code` (migration 0005).
-- Plus d'adresse postale complète : `adresse` disparaît. Reprise depuis
-- l'ancienne `adresse` (texte libre) :
--   1. un code postal FR (5 chiffres isolés), sinon CH (4 chiffres isolés),
--      le dernier trouvé, devient `zip_code` ; `ville` reçoit le texte qui le
--      suit jusqu'à la virgule suivante, ou, si rien ne le suit, le segment
--      qui le précède (« Thyez, 74300 ») s'il ne ressemble pas à une rue ;
--   2. sans code postal, la valeur entière devient `ville` si elle ne
--      ressemble pas à une rue (aucun chiffre, aucun mot de voie) ;
--   3. sinon, les deux restent NULL.
-- Les sauts de ligne de l'ancienne valeur comptent comme des virgules.
-- SQLite n'a ni expressions régulières ni `ADD COLUMN IF NOT EXISTS` : la
-- recherche du code postal parcourt les positions par CTE récursive, et
-- l'application unique est garantie par le migrateur (`_sqlx_migrations`).

ALTER TABLE users ADD COLUMN ville TEXT;
ALTER TABLE users ADD COLUMN zip_code TEXT;

-- ------------------------------------------------------------------
-- 1. Texte de travail : sauts de ligne → virgules, bords nettoyés
-- ------------------------------------------------------------------
CREATE TEMP TABLE location_work AS
SELECT id,
       TRIM(REPLACE(REPLACE(COALESCE(adresse, ''), char(13), ''), char(10), ', '), ' ,') AS a
  FROM users;

-- ------------------------------------------------------------------
-- 2. Code postal : 5 chiffres préférés à 4, dernière occurrence
--    (le numéro de rue vient avant le code postal)
-- ------------------------------------------------------------------
CREATE TEMP TABLE location_zip AS
WITH RECURSIVE pos(id, a, p) AS (
    SELECT id, a, 1 FROM location_work WHERE a <> ''
    UNION ALL
    SELECT id, a, p + 1 FROM pos WHERE p < length(a)
),
matches(id, p, len) AS (
    SELECT id, p, 5 FROM pos
     WHERE substr(a, p, 5) GLOB '[0-9][0-9][0-9][0-9][0-9]'
       AND (p = 1 OR substr(a, p - 1, 1) NOT GLOB '[0-9]')
       AND substr(a, p + 5, 1) NOT GLOB '[0-9]'
    UNION ALL
    SELECT id, p, 4 FROM pos
     WHERE substr(a, p, 4) GLOB '[0-9][0-9][0-9][0-9]'
       AND (p = 1 OR substr(a, p - 1, 1) NOT GLOB '[0-9]')
       AND substr(a, p + 4, 1) NOT GLOB '[0-9]'
)
SELECT id, p, len
  FROM (SELECT id, p, len, ROW_NUMBER() OVER (PARTITION BY id ORDER BY len DESC, p DESC) AS rn FROM matches)
 WHERE rn = 1;

-- ------------------------------------------------------------------
-- 3. Segments candidats à la ville : après le code postal (jusqu'à la
--    virgule), avant le code postal (depuis la dernière virgule), ou la
--    valeur entière s'il n'y a pas de code postal
-- ------------------------------------------------------------------
CREATE TEMP TABLE location_candidates AS
SELECT id,
       has_zip,
       zip,
       TRIM(substr(after_zip, 1, instr(after_zip || ',', ',') - 1)) AS after_city,
       -- rtrim(before, <caractères de before hors virgules>) garde tout jusqu'à
       -- la dernière virgule incluse ; le retirer laisse le dernier segment.
       TRIM(REPLACE(before_zip, RTRIM(before_zip, REPLACE(before_zip, ',', '')), '')) AS before_city,
       whole
  FROM (SELECT w.id,
               z.p IS NOT NULL AS has_zip,
               substr(w.a, z.p, z.len) AS zip,
               LTRIM(substr(w.a, z.p + z.len), ' ,') AS after_zip,
               RTRIM(substr(w.a, 1, z.p - 1), ' ,') AS before_zip,
               w.a AS whole
          FROM location_work w
          LEFT JOIN location_zip z ON z.id = w.id);

-- ------------------------------------------------------------------
-- 4. Mots de voie : un segment qui en contient un (ou un chiffre) est une
--    rue, pas une ville
-- ------------------------------------------------------------------
CREATE TEMP TABLE location_street_words (word TEXT NOT NULL);
INSERT INTO location_street_words (word) VALUES
    ('rue'), ('avenue'), ('av'), ('bd'), ('boulevard'), ('chemin'), ('allée'), ('allee'), ('place'),
    ('impasse'), ('route'), ('quai'), ('cours'), ('square'), ('voie'), ('passage'), ('sentier'),
    ('chaussée'), ('chaussee'), ('faubourg'), ('lieu-dit'), ('résidence'), ('residence'), ('hameau'),
    ('lotissement'), ('bis'), ('ter');

CREATE TEMP TABLE location_result AS
SELECT id,
       NULLIF(zip, '') AS zip_code,
       NULLIF(CASE
                  WHEN has_zip AND after_city <> '' THEN after_city
                  WHEN has_zip AND NOT before_is_street THEN before_city
                  WHEN NOT has_zip AND NOT whole_is_street THEN whole
              END, '') AS ville
  FROM (SELECT c.*,
               (c.before_city GLOB '*[0-9]*' OR EXISTS (
                   SELECT 1 FROM location_street_words s
                    WHERE ' ' || lower(REPLACE(REPLACE(c.before_city, ',', ' '), '.', ' ')) || ' '
                          LIKE '% ' || s.word || ' %')) AS before_is_street,
               (c.whole GLOB '*[0-9]*' OR EXISTS (
                   SELECT 1 FROM location_street_words s
                    WHERE ' ' || lower(REPLACE(REPLACE(c.whole, ',', ' '), '.', ' ')) || ' '
                          LIKE '% ' || s.word || ' %')) AS whole_is_street
          FROM location_candidates c);

-- ------------------------------------------------------------------
-- 5. Écriture, suppression de `adresse` et des tables de travail
-- ------------------------------------------------------------------
UPDATE users
   SET ville    = (SELECT r.ville FROM location_result r WHERE r.id = users.id),
       zip_code = (SELECT r.zip_code FROM location_result r WHERE r.id = users.id);

ALTER TABLE users DROP COLUMN adresse;

DROP TABLE location_result;
DROP TABLE location_street_words;
DROP TABLE location_candidates;
DROP TABLE location_zip;
DROP TABLE location_work;
