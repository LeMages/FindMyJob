//! Démarrage de l'application : chargement du `.env`, mode d'exécution,
//! initialisation de l'état (dossiers, base, nettoyage) et création de la
//! fenêtre principale.

pub mod env_file;
pub mod initialize;
pub mod main_window;
pub mod run_mode;

pub use initialize::{initialize, Startup};
