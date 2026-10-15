-- FindMyJob : adresse du profil en un seul champ libre (migration 0002).
-- `adresse` (la rue) et `code_postal` fusionnent dans `adresse`, texte libre
-- affiché tel quel dans les documents. Reprise : rue et code postal joints par
-- une espace, une valeur vide ou NULL est ignorée ; rien de renseigné → NULL.

UPDATE users
   SET adresse = NULLIF(TRIM(COALESCE(TRIM(adresse), '') || ' ' || COALESCE(TRIM(code_postal), '')), '');

ALTER TABLE users DROP COLUMN code_postal;
