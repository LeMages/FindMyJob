//! Fonctions de service : la logique métier qui dépasse un accès base
//! (règles, orchestration, fichiers), appelée par les commandes Tauri.

pub mod api_key;
pub mod candidature_validation;
pub mod candidatures;
pub mod claude_target;
pub mod generation;
pub mod profile_language;
pub mod scraping;
pub mod sessions;
pub mod settings;
pub mod user_errors;
