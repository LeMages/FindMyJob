//! Ré-exports publics de la couche d'accès aux données PostgreSQL de l'agent.

pub mod candidatures;
pub mod common;
pub mod config;
pub mod contacts;
pub mod document_versions;
pub mod documents;
pub mod experiences;
pub mod formations;
pub mod init;
pub mod interviews;
pub mod joins;
pub mod langues;
pub mod profile;
pub mod projets;
pub mod sessions;
pub mod settings;
pub mod skills;
pub mod status_history;
pub mod timeline_events;
pub mod users;

pub use config::DbConfig;
pub use init::init_db;
