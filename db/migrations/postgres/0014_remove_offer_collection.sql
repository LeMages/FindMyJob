-- Retrait de la collecte automatique d'offres (scraping, filtrage, scoring
-- LLM, onglet Marché) : chaque offre est désormais ajoutée par son lien et
-- vit uniquement dans sa candidature (url_offre, description_offre).
-- L'ancien pipeline reste consultable au tag Git archive/collecte-automatique.

-- Le lien candidature -> offre est supprimé en premier : c'est la seule clé
-- étrangère qui référence search.offers (son index part avec la colonne).
ALTER TABLE search.candidatures DROP COLUMN offer_id;

DROP TABLE search.search_queries;
DROP TABLE search.offers;

-- Colonnes qui ne servaient qu'au scoring (contexte, zone) et à l'affichage
-- du Marché (min_score, dont la contrainte CHECK part avec la colonne).
ALTER TABLE search.sessions
    DROP COLUMN min_score,
    DROP COLUMN contexte,
    DROP COLUMN zone;
