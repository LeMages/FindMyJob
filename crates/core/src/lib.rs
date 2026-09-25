//! Crate `findmyjob-core` : logique métier pure (modèles, appels LLM,
//! scraping), sans dépendance à une base de données ni à Tauri. La
//! persistance, la génération de documents et la gestion de la clé API
//! vivent dans les crates qui consomment `core` (ex. `findmyjob-agent`).

pub mod error;
pub mod llm;
pub mod models;
pub mod scraping;
pub mod text;
