//! Binaire unique des tests d'intégration de `findmyjob` : chaque fichier
//! de ce dossier est un module, `shared/` regroupe les aides communes.
//! Un seul binaire par crate limite l'espace disque et le temps d'édition
//! de liens ; tout nouveau test d'intégration est un module déclaré ici.

mod shared;

mod candidature_deletion;
mod db_candidatures;
mod db_contacts;
mod db_init;
mod db_migration_adresse;
mod db_profile_entries;
mod db_sessions_documents;
mod db_skills_langues;
mod db_timeline_events;
mod db_users;
mod end_to_end;
mod generation;
mod offer_import;
mod profile_lifecycle;
mod services_api_key;
mod services_files;
mod sessions_lifecycle;
mod startup_run_mode;
