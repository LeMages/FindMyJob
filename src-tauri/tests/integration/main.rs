//! Binaire unique des tests d'intégration de `findmyjob` : chaque fichier
//! de ce dossier est un module, `common/` regroupe les fixtures partagées
//! de la logique pure, `shared/` les aides communes (état, LLM factice,
//! serveur local). Un seul binaire limite l'espace disque et le temps
//! d'édition de liens ; tout nouveau test d'intégration est un module
//! déclaré ici.

mod common;
mod shared;

mod candidature_deletion;
mod db_candidatures;
mod db_contacts;
mod db_init;
mod db_migration_adresse;
mod db_migration_ville;
mod db_profile_entries;
mod db_sessions_documents;
mod db_skills_langues;
mod db_timeline_events;
mod db_users;
mod docgen_budgets;
mod docgen_dates;
mod docgen_lint;
mod docgen_prose_guidance;
mod docgen_selection;
mod docgen_sentinel;
mod docgen_text_diff;
mod docgen_translation;
mod end_to_end;
mod generation;
mod generation_budgets;
mod llm_schema_size;
mod offer_import;
mod profile_lifecycle;
mod scraping_contract_type;
mod scraping_fetch_rules;
mod scraping_finalize;
mod scraping_json_ld;
mod scraping_page_text;
mod services_api_key;
mod services_files;
mod sessions_lifecycle;
mod startup_run_mode;
mod templatize_markup;
mod templatize_retries;
mod templatize_validation;
mod text_conversion;
