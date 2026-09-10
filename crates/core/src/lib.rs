//! Crate `findmyjob-core` : logique métier pure (base de données, modèles,
//! génération de documents, appels LLM, scraping), sans dépendance à Tauri.

pub mod db;
pub mod documents;
pub mod error;
pub mod llm;
pub mod models;
pub mod scraping;
pub mod security;
