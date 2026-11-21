-- FindMyJob : les centres d'intérêt quittent le profil (migration 0009).
-- Une ligne « Intérêts » d'une référence est désormais un texte en dur
-- de la référence (data.texts), traduit et mis en cache comme les autres.

ALTER TABLE users DROP COLUMN interets;
