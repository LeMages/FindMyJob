//! Coffre adossé au trousseau du système (crate `keyring`) : Gestionnaire
//! d'identification sous Windows, Trousseau sous macOS, Secret Service sous
//! Linux. Un trousseau indisponible est une erreur explicite — aucun repli
//! sur un fichier.

// Importation des crates externes
use keyring::{Entry, Error as KeyringError};

// Importation des modules internes
use crate::secrets::store::SecretStore;
use findmyjob_core::error::AppError;

/// Opération sur le trousseau, nommée dans les messages d'erreur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyringAction {
    Read,
    Write,
    Delete,
}

impl KeyringAction {
    /// Verbe de l'opération, pour les messages d'erreur.
    fn verb(self) -> &'static str {
        match self {
            KeyringAction::Read => "lire",
            KeyringAction::Write => "enregistrer",
            KeyringAction::Delete => "supprimer",
        }
    }
}

/// Entrée du trousseau du système identifiée par un service et un compte.
///
/// # Attributes
///
/// * `service` - Service de l'entrée (identifiant de l'application).
/// * `account` - Compte de l'entrée (nom du secret).
pub struct KeyringStore {
    service: String,
    account: String,
}

impl KeyringStore {
    /// Construit le coffre sans toucher au trousseau : il n'est ouvert qu'au
    /// premier accès.
    ///
    /// # Arguments
    ///
    /// * `service` - Service de l'entrée (identifiant de l'application).
    /// * `account` - Compte de l'entrée (nom du secret).
    pub fn new(service: &str, account: &str) -> Self {
        Self { service: service.to_string(), account: account.to_string() }
    }

    /// Ouvre l'entrée du trousseau.
    ///
    /// # Arguments
    ///
    /// * `action` - Opération en cours, pour le message d'erreur.
    ///
    /// # Returns
    ///
    /// L'entrée, ou `AppError::Config` si le trousseau est indisponible.
    fn entry(&self, action: KeyringAction) -> Result<Entry, AppError> {
        // `Entry::new` ne rapporte qu'un `NoDefaultStore` générique quand le
        // trousseau n'a pas pu s'ouvrir : la cause réelle est dans `store_status`.
        if let Err(cause) = Entry::store_status() {
            return Err(map_keyring_error(cause, action));
        }
        Entry::new(&self.service, &self.account).map_err(|e| map_keyring_error(&e, action))
    }
}

impl SecretStore for KeyringStore {
    fn read(&self) -> Result<Option<String>, AppError> {
        match self.entry(KeyringAction::Read)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(e) => Err(map_keyring_error(&e, KeyringAction::Read)),
        }
    }

    fn write(&self, value: &str) -> Result<(), AppError> {
        self.entry(KeyringAction::Write)?
            .set_password(value)
            .map_err(|e| map_keyring_error(&e, KeyringAction::Write))
    }

    fn delete(&self) -> Result<(), AppError> {
        match self.entry(KeyringAction::Delete)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(e) => Err(map_keyring_error(&e, KeyringAction::Delete)),
        }
    }
}

/// Traduit une erreur du trousseau en message français. Le détail technique
/// n'est repris que pour les erreurs d'accès, dont le texte ne contient
/// jamais le secret ; les erreurs sur la donnée elle-même restent génériques.
///
/// # Arguments
///
/// * `error` - Erreur renvoyée par `keyring`.
/// * `action` - Opération en cours.
///
/// # Returns
///
/// `AppError::Config` explicite.
pub fn map_keyring_error(error: &KeyringError, action: KeyringAction) -> AppError {
    let verb = action.verb();
    match error {
        KeyringError::NoStorageAccess(_) | KeyringError::PlatformFailure(_) | KeyringError::NoDefaultStore => {
            AppError::Config(format!(
                "Trousseau du système indisponible : impossible de {verb} la clé API Anthropic ({error}). \
                 Sous Linux, un service Secret Service (GNOME Keyring, KWallet) doit être actif et déverrouillé."
            ))
        }
        KeyringError::Invalid(..) => AppError::Config(format!(
            "Trousseau du système non pris en charge sur cette plateforme : impossible de {verb} la clé API Anthropic."
        )),
        _ => AppError::Config(format!(
            "Le trousseau du système a refusé l'opération : impossible de {verb} la clé API Anthropic."
        )),
    }
}
