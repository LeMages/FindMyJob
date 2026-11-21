-- FindMyJob : la localisation de l'expéditeur passe du profil à la session
-- (migration 0008). L'ordre est impératif : colonnes de session ajoutées,
-- valeur du profil copiée dans TOUTES les sessions existantes, puis
-- seulement suppression des colonnes du profil.

ALTER TABLE sessions ADD COLUMN ville TEXT;
ALTER TABLE sessions ADD COLUMN zip_code TEXT;

-- Sans profil (users vide), les sous-requêtes donnent NULL : rien à copier.
UPDATE sessions
   SET ville    = (SELECT u.ville FROM users u WHERE u.id = 1),
       zip_code = (SELECT u.zip_code FROM users u WHERE u.id = 1);

ALTER TABLE users DROP COLUMN ville;
ALTER TABLE users DROP COLUMN zip_code;
