//! Ré-exports publics de la couche d'accès aux données. Chaque sous-module
//! porte la logique de persistance d'une seule table (ou d'un helper partagé).

pub mod candidatures;
pub mod common;
pub mod contacts;
pub mod documents;
pub mod experiences;
pub mod formations;
pub mod init;
pub mod langues;
pub mod profile;
pub mod projets;
pub mod sessions;
pub mod skills;
pub mod status_history;
pub mod users;

pub use init::{list_tables, resolve_data_dir, resolve_db_path, DbState};
