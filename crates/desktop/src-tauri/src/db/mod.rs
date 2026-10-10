//! Couche d'accès à la base SQLite de l'application : un module par table,
//! `joins` pour les requêtes inter-tables, `profile` pour l'assemblage du profil.

pub mod candidatures;
pub mod common;
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
pub mod transaction;
pub mod users;
