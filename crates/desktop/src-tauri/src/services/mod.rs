//! Fonctions de service : la logique métier qui dépasse un accès base
//! (règles, orchestration, fichiers), appelée par les commandes Tauri.

pub mod candidature_validation;
pub mod candidatures;
pub mod generation;
pub mod scraping;
pub mod sessions;
pub mod settings;
