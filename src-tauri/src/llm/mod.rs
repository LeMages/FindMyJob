//! Ré-exports publics de la couche d'intégration avec l'API Claude : client
//! HTTP générique et logique d'analyse de style (étape 0 du pipeline IA).

pub mod client;
pub mod style;

pub use client::call_claude;
pub use style::analyze_style;
