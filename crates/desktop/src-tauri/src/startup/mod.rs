//! Démarrage de l'application : chargement du `.env` et initialisation de
//! l'état (dossiers, base, nettoyage).

pub mod env_file;
pub mod initialize;

pub use initialize::initialize;
