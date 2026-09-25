-- Champs HTML/Markdown d'origine ayant servi à construire `description`
-- (texte brut), pour pouvoir rejouer la conversion sans réseau :
-- {description, profile, recruitment_process, source: "api_v1" | "algolia"}.
-- NULL pour les offres insérées avant cette migration.
ALTER TABLE search.offers ADD COLUMN description_raw JSONB;
