-- FindMyJob : contexte détaillé des expériences et des projets (migration 0010).
-- Texte libre facultatif, jamais rendu sur un document : il est fourni au
-- LLM pour le choix des projets (contexte des projets) et pour la lettre.
-- Colonne nullable sans défaut : les lignes existantes restent à NULL, ce
-- qui reproduit le comportement antérieur.

ALTER TABLE experiences ADD COLUMN contexte TEXT;
ALTER TABLE projets ADD COLUMN contexte TEXT;
