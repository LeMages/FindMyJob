-- FindMyJob : sessions terminées par une offre acceptée (migration 0011).
-- Données seulement, schéma inchangé. Désormais, accepter une offre passe
-- la session à « termine » dans la même transaction ; cette migration
-- applique la même règle aux sessions existantes qui contiennent déjà une
-- candidature acceptée.

UPDATE sessions
SET statut = 'termine'
WHERE EXISTS (SELECT 1 FROM candidatures c WHERE c.session_id = sessions.id AND c.status = 'accepte');
