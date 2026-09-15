//! Gestion de la clé API Anthropic : `.env` en développement (chargé par
//! `dotenvy`, voir `main.rs`), trousseau de l'OS (`keyring`) en production.
//! La clé n'est jamais persistée en base de données.

use findmyjob_core::error::AppError;

#[cfg(not(debug_assertions))]
const KEYRING_SERVICE: &str = "FindMyJob";
#[cfg(not(debug_assertions))]
const KEYRING_USER: &str = "anthropic_api_key";

#[cfg(not(debug_assertions))]
fn keyring_entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| AppError::Config(format!("Trousseau OS inaccessible : {e}")))
}

pub fn get_api_key() -> Result<Option<String>, AppError> {
    #[cfg(debug_assertions)]
    { Ok(std::env::var("ANTHROPIC_API_KEY").ok()) }
    #[cfg(not(debug_assertions))]
    {
        match keyring_entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Config(format!("Lecture du trousseau OS impossible : {e}"))),
        }
    }
}

pub fn set_api_key(key: &str) -> Result<(), AppError> {
    #[cfg(debug_assertions)]
    { let _ = key; Err(AppError::Config("En développement, configurez ANTHROPIC_API_KEY dans le fichier .env.".to_string())) }
    #[cfg(not(debug_assertions))]
    { keyring_entry()?.set_password(key).map_err(|e| AppError::Config(format!("Écriture du trousseau OS impossible : {e}"))) }
}

pub fn delete_api_key() -> Result<(), AppError> {
    #[cfg(debug_assertions)]
    { Err(AppError::Config("En développement, configurez ANTHROPIC_API_KEY dans le fichier .env.".to_string())) }
    #[cfg(not(debug_assertions))]
    {
        match keyring_entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Config(format!("Suppression du trousseau OS impossible : {e}"))),
        }
    }
}

pub fn has_api_key() -> bool {
    matches!(get_api_key(), Ok(Some(_)))
}

pub fn key_source() -> &'static str {
    if cfg!(debug_assertions) { "env" } else { "keyring" }
}
