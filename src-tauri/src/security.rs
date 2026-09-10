//! Gestion de la clé API Anthropic : `.env` en développement (chargé par
//! `dotenvy`, voir `lib.rs`), trousseau de l'OS (`keyring`) en production.
//! La clé n'est jamais persistée en base de données.

// Importation des modules internes
use crate::error::AppError;

/// Nom du service utilisé pour l'entrée du trousseau OS.
#[cfg(not(debug_assertions))]
const KEYRING_SERVICE: &str = "FindMyJob";
/// Nom d'utilisateur logique associé à l'entrée du trousseau OS.
#[cfg(not(debug_assertions))]
const KEYRING_USER: &str = "anthropic_api_key";

/// Ouvre l'entrée du trousseau OS pour la clé API Anthropic.
#[cfg(not(debug_assertions))]
fn keyring_entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| AppError::Config(format!("Trousseau OS inaccessible : {e}")))
}

/// Récupère la clé API Anthropic configurée, si elle existe.
///
/// # Returns
///
/// La clé API si elle est configurée (via `.env` en développement, via le
/// trousseau OS en production), `None` sinon.
pub fn get_api_key() -> Result<Option<String>, AppError> {
    #[cfg(debug_assertions)]
    {
        Ok(std::env::var("ANTHROPIC_API_KEY").ok())
    }
    #[cfg(not(debug_assertions))]
    {
        match keyring_entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Config(format!(
                "Lecture du trousseau OS impossible : {e}"
            ))),
        }
    }
}

/// Enregistre (ou remplace) la clé API Anthropic.
///
/// En développement, la clé est censée être définie dans `.env` : cette
/// fonction échoue explicitement plutôt que de réécrire ce fichier.
///
/// # Arguments
///
/// * `key` - Nouvelle valeur de la clé API.
///
/// # Returns
///
/// `Ok(())` si l'écriture a réussi.
pub fn set_api_key(key: &str) -> Result<(), AppError> {
    #[cfg(debug_assertions)]
    {
        let _ = key;
        Err(AppError::Config(
            "En développement, configurez ANTHROPIC_API_KEY dans le fichier .env.".to_string(),
        ))
    }
    #[cfg(not(debug_assertions))]
    {
        keyring_entry()?
            .set_password(key)
            .map_err(|e| AppError::Config(format!("Écriture du trousseau OS impossible : {e}")))
    }
}

/// Supprime la clé API Anthropic enregistrée.
///
/// En développement, la clé est censée être définie dans `.env` : cette
/// fonction échoue explicitement plutôt que d'agir sur ce fichier.
///
/// # Returns
///
/// `Ok(())` si la suppression a réussi (y compris si aucune clé n'existait).
pub fn delete_api_key() -> Result<(), AppError> {
    #[cfg(debug_assertions)]
    {
        Err(AppError::Config(
            "En développement, configurez ANTHROPIC_API_KEY dans le fichier .env.".to_string(),
        ))
    }
    #[cfg(not(debug_assertions))]
    {
        match keyring_entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Config(format!(
                "Suppression du trousseau OS impossible : {e}"
            ))),
        }
    }
}

/// Indique si une clé API est actuellement configurée (sans la révéler).
///
/// # Returns
///
/// `true` si une clé API est disponible, `false` sinon.
pub fn has_api_key() -> bool {
    matches!(get_api_key(), Ok(Some(_)))
}

/// Source de la clé API actuellement active, pour affichage dans les
/// paramètres (`"env"` en développement, `"keyring"` en production).
///
/// # Returns
///
/// `"env"` ou `"keyring"` selon le mode de compilation.
pub fn key_source() -> &'static str {
    if cfg!(debug_assertions) {
        "env"
    } else {
        "keyring"
    }
}
