-- Trace le message (mail ou message de candidature) qui a accompagné un
-- changement de statut, pour permettre de retrouver ce qui a été envoyé.
-- NULL quand le changement n'est associé à aucun message (ex. « Postuler
-- sans message », abandon, création).

ALTER TABLE status_history ADD COLUMN content TEXT;
