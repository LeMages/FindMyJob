//! Crate `findmyjob-core` : logique métier pure (modèles, appels LLM,
//! scraping, traces), sans dépendance à une base de données ni à Tauri. La
//! persistance, la génération de documents et le chargement de la clé API
//! vivent dans l'application desktop (`crates/desktop/src-tauri`).

pub mod docgen;
pub mod error;
pub mod llm;
pub mod models;
pub mod scraping;
pub mod text;
pub mod trace_log;
