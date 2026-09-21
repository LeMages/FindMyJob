-- Seuil de score minimum d'affichage des offres (onglet Marché), propre à
-- chaque session. Borné à [0.05, 1] : 0 afficherait des offres non pertinentes.
ALTER TABLE search.sessions
    ADD COLUMN min_score REAL NOT NULL DEFAULT 0.5
    CONSTRAINT sessions_min_score_range CHECK (min_score >= 0.05 AND min_score <= 1);
