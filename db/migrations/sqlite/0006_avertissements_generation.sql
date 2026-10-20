-- FindMyJob : avertissements de la génération d'une candidature (migration 0006).
-- Liste JSON de messages pour l'utilisateur (document qui dépasse son nombre
-- de pages…), affichée dans la fiche de la candidature. '[]' = aucun.

ALTER TABLE candidatures ADD COLUMN generation_warnings TEXT NOT NULL DEFAULT '[]';
