-- FindMyJob : descriptions du profil au format « - élément » (migration 0013).
-- Données seulement, schéma inchangé. Désormais, une ligne qui commence par
-- « - » est un élément de liste, et une ligne sans tiret est du texte (intro
-- ou suite d'élément). Avant, chaque ligne non vide d'une description
-- d'expérience, de projet ou de formation devenait un élément du CV, après
-- retrait des symboles de tête (•, -, *, –, —). Pour garder le même rendu,
-- chaque ligne non vide devient « - <contenu> », où <contenu> est
-- exactement l'élément que l'ancien découpage produisait :
--   - symboles de tête retirés, puis espaces autour ;
--   - seule exception : une ligne « -<chiffre>… » (« -30 % ») garde son
--     signe, que l'ancien découpage retirait à tort ;
--   - les lignes vides, ou vides une fois les symboles retirés, disparaissent ;
--   - un « • » au milieu d'une ligne reste un caractère.
-- Rejouée sur des données déjà migrées, elle ne change rien.

-- ----------------------------------------------------------------------
-- 1. Découpage de toutes les descriptions en lignes numérotées
-- ----------------------------------------------------------------------
-- Les « \r » (texte collé depuis Windows) sont retirés avant le découpage.
-- Un « \n » final ajouté garantit que la dernière ligne est découpée.
CREATE TEMP TABLE migration_lignes (
    tbl   TEXT    NOT NULL,
    id    INTEGER NOT NULL,
    n     INTEGER NOT NULL,
    ligne TEXT    NOT NULL
);

WITH RECURSIVE decoupe(tbl, id, n, ligne, reste) AS (
    SELECT 'experiences', id, 0, '', replace(description, char(13), '') || char(10)
    FROM experiences WHERE description IS NOT NULL
    UNION ALL
    SELECT 'projets', id, 0, '', replace(description, char(13), '') || char(10)
    FROM projets WHERE description IS NOT NULL
    UNION ALL
    SELECT 'formations', id, 0, '', replace(description, char(13), '') || char(10)
    FROM formations WHERE description IS NOT NULL
    UNION ALL
    SELECT tbl, id, n + 1,
           substr(reste, 1, instr(reste, char(10)) - 1),
           substr(reste, instr(reste, char(10)) + 1)
    FROM decoupe
    WHERE reste <> ''
)
INSERT INTO migration_lignes (tbl, id, n, ligne)
SELECT tbl, id, n, trim(ligne, ' ' || char(9) || char(160))
FROM decoupe
WHERE n > 0;

-- ----------------------------------------------------------------------
-- 2. Conversion de chaque ligne en élément « - <contenu> »
-- ----------------------------------------------------------------------
-- Même retrait que l'ancien découpage : symboles de tête, puis espaces.
-- Une ligne déjà migrée (« - x ») redonne « - x » : rejouable sans effet.
UPDATE migration_lignes
SET ligne = CASE
    WHEN ligne GLOB '-[0-9]*' THEN '- ' || ligne
    WHEN trim(ltrim(ligne, '•*-–—'), ' ' || char(9) || char(160)) = '' THEN ''
    ELSE '- ' || trim(ltrim(ligne, '•*-–—'), ' ' || char(9) || char(160))
END;

-- ----------------------------------------------------------------------
-- 3. Recomposition, dans l'ordre des lignes (aucune ligne = NULL)
-- ----------------------------------------------------------------------
UPDATE experiences
SET description = (
    SELECT group_concat(ligne, char(10) ORDER BY n)
    FROM migration_lignes
    WHERE tbl = 'experiences' AND migration_lignes.id = experiences.id AND ligne <> ''
)
WHERE description IS NOT NULL;

UPDATE projets
SET description = (
    SELECT group_concat(ligne, char(10) ORDER BY n)
    FROM migration_lignes
    WHERE tbl = 'projets' AND migration_lignes.id = projets.id AND ligne <> ''
)
WHERE description IS NOT NULL;

UPDATE formations
SET description = (
    SELECT group_concat(ligne, char(10) ORDER BY n)
    FROM migration_lignes
    WHERE tbl = 'formations' AND migration_lignes.id = formations.id AND ligne <> ''
)
WHERE description IS NOT NULL;

DROP TABLE migration_lignes;
