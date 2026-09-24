-- Rattache une candidature créée depuis le Marché (bouton « Postuler » sur
-- une ligne de MarchePanel.vue) à l'offre d'origine — nullable : les
-- candidatures créées manuellement n'ont pas d'offre source. Pas de
-- ON DELETE : une offre référencée par une candidature ne doit jamais être
-- supprimée.
ALTER TABLE search.candidatures
    ADD COLUMN offer_id BIGINT REFERENCES search.offers(id);

CREATE INDEX idx_candidatures_offer_id ON search.candidatures(offer_id);
