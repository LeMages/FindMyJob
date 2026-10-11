-- FindMyJob : `adresse` devient le seul champ de lieu du profil (migration 0003).
-- Texte libre (adresse complète, ville seule…), affiché tel quel dans les
-- documents ; `ville` disparaît. Reprise : `adresse` reçoit l'ancienne `ville`
-- (vide → NULL) ; l'adresse fusionnée par la migration 0002 n'est pas reprise.

UPDATE users
   SET adresse = NULLIF(TRIM(ville), '');

ALTER TABLE users DROP COLUMN ville;
