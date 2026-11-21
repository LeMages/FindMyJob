-- FindMyJob : format des listes des CV par session (migration 0012).
-- Symbole placé devant chaque élément de liste des descriptions dans les CV
-- générés de la session : 'puce' (•) ou 'tiret' (-). Le défaut est une
-- constante (pas une date) : les sessions existantes gardent la puce, le
-- symbole des références actuelles.

ALTER TABLE sessions ADD COLUMN format_liste TEXT NOT NULL DEFAULT 'puce' CHECK (format_liste IN ('puce', 'tiret'));
