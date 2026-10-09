-- Refonte du pipeline de génération de documents : chaque référence (CV, LM)
-- est convertie une fois pour toutes, à l'upload, en un template Typst qui lit
-- ses données depuis un JSON au schéma canonique (template.typ) et en un jeu
-- de données d'exemple (example_data.json). L'analyse de style LLM n'a plus
-- d'usage : ses colonnes sont supprimées. Une référence sans template n'est
-- plus « prête » tant qu'elle n'a pas été re-templatisée.
ALTER TABLE search.documents
    DROP COLUMN style_analysis,
    ADD COLUMN template_path text,
    ADD COLUMN example_data_path text,
    ADD COLUMN page_count integer;

ALTER TABLE search.document_versions
    DROP COLUMN style_analysis,
    ADD COLUMN template_path text,
    ADD COLUMN example_data_path text,
    ADD COLUMN page_count integer;

-- Champs affichés par les documents de référence, sans source en base
-- jusqu'ici : adresse postale de l'expéditeur, centres d'intérêt (une ligne =
-- un item), détails d'une formation (une ligne = un détail).
ALTER TABLE profile.users
    ADD COLUMN adresse text,
    ADD COLUMN code_postal text,
    ADD COLUMN interets text;

ALTER TABLE profile.formations
    ADD COLUMN description text;
