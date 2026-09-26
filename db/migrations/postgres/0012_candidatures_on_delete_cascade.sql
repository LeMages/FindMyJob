-- Supprimer une candidature (bouton « Supprimer » au statut « à postuler »,
-- ou échec de génération des documents juste après la création) emporte
-- toutes les lignes qui la référencent. Les FK internes
-- (status_history.interview_id, interviews.contact_id) restent sans
-- ON DELETE : elles sont vérifiées en fin d'instruction, quand les lignes
-- liées ont déjà été supprimées par cascade.
ALTER TABLE search.status_history
    DROP CONSTRAINT status_history_candidature_id_fkey,
    ADD CONSTRAINT status_history_candidature_id_fkey
        FOREIGN KEY (candidature_id) REFERENCES search.candidatures(id) ON DELETE CASCADE;

ALTER TABLE search.contacts
    DROP CONSTRAINT contacts_candidature_id_fkey,
    ADD CONSTRAINT contacts_candidature_id_fkey
        FOREIGN KEY (candidature_id) REFERENCES search.candidatures(id) ON DELETE CASCADE;

ALTER TABLE search.interviews
    DROP CONSTRAINT interviews_candidature_id_fkey,
    ADD CONSTRAINT interviews_candidature_id_fkey
        FOREIGN KEY (candidature_id) REFERENCES search.candidatures(id) ON DELETE CASCADE;
