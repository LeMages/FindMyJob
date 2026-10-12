//! Tests de la clé API Anthropic : source selon le mode (`.env` en DEV,
//! trousseau en PROD), clé absente, enregistrement et suppression, refus en
//! mode DEV, masquage. Le trousseau est un faux en mémoire : aucun test ne
//! touche au trousseau du système.

// Importation des modules standard
use std::sync::Arc;

// Importation des crates externes
use keyring::Error as KeyringError;

// Importation des modules internes
use crate::shared::fake_secret_store;
use fake_secret_store::FakeSecretStore;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::ApiKey;
use findmyjob_core::models::{ApiKeySource, ApiKeyStatus};
use findmyjob_lib::secrets::keyring_store::{map_keyring_error, KeyringAction};
use findmyjob_lib::services::api_key::ApiKeyAccess;
use findmyjob_lib::startup::run_mode::RunMode;

const ENV_KEY: &str = "sk-ant-env-key-ENV1";
const STORED_KEY: &str = "sk-ant-stored-key-KR42";

fn env_with_key() -> Option<String> {
    Some(ENV_KEY.to_string())
}

fn env_empty() -> Option<String> {
    None
}

fn env_blank() -> Option<String> {
    Some("   ".to_string())
}

/// Accès à la clé sur un faux trousseau partagé avec le test.
fn access(mode: RunMode, store: &Arc<FakeSecretStore>, read_env: fn() -> Option<String>) -> ApiKeyAccess {
    ApiKeyAccess::new(mode, store.clone(), read_env)
}

/// Message d'une `AppError::Config` attendue.
fn config_message(result: Result<ApiKey, AppError>) -> String {
    match result {
        Err(AppError::Config(message)) => message,
        other => panic!("AppError::Config attendue, reçu {other:?}"),
    }
}

// ======================================================================
// Source de la clé selon le mode
// ======================================================================

#[test]
fn dev_lit_l_environnement_sans_consulter_le_trousseau() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Dev, &store, env_with_key);

    assert_eq!(access.resolve().unwrap().expose(), ENV_KEY);
    let status = access.status().unwrap();
    assert_eq!(status, ApiKeyStatus { present: true, source: ApiKeySource::Env, last4: Some("ENV1".to_string()) });
    assert_eq!(store.accesses(), 0);
}

#[test]
fn prod_lit_le_trousseau_et_ignore_l_environnement() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Prod, &store, env_with_key);

    assert_eq!(access.resolve().unwrap().expose(), STORED_KEY);
    let status = access.status().unwrap();
    assert_eq!(status, ApiKeyStatus { present: true, source: ApiKeySource::Keyring, last4: Some("KR42".to_string()) });
}

#[test]
fn prod_sans_cle_ignore_quand_meme_l_environnement() {
    let store = Arc::new(FakeSecretStore::default());
    let access = access(RunMode::Prod, &store, env_with_key);

    let message = config_message(access.resolve());
    assert!(message.starts_with("Aucune clé API Anthropic configurée."), "{message}");
    assert!(message.contains("Paramètres"), "{message}");
    assert!(!access.status().unwrap().present);
}

#[test]
fn dev_sans_cle_indique_le_fichier_env() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Dev, &store, env_empty);

    let message = config_message(access.resolve());
    assert!(message.starts_with("Aucune clé API Anthropic configurée."), "{message}");
    assert!(message.contains(".env"), "{message}");
    assert_eq!(access.status().unwrap(), ApiKeyStatus { present: false, source: ApiKeySource::Env, last4: None });
    assert_eq!(store.accesses(), 0);
}

#[test]
fn une_cle_faite_d_espaces_est_absente() {
    let store = Arc::new(FakeSecretStore::default());
    assert!(matches!(access(RunMode::Dev, &store, env_blank).resolve(), Err(AppError::Config(_))));
}

// ======================================================================
// Enregistrement et suppression
// ======================================================================

#[test]
fn prod_enregistre_la_cle_nettoyee() {
    let store = Arc::new(FakeSecretStore::default());
    let access = access(RunMode::Prod, &store, env_empty);

    let status = access.save("  sk-ant-saisie-AB12 \n").unwrap();
    assert_eq!(status, ApiKeyStatus { present: true, source: ApiKeySource::Keyring, last4: Some("AB12".to_string()) });
    assert_eq!(store.value().as_deref(), Some("sk-ant-saisie-AB12"));
    assert_eq!(access.resolve().unwrap().expose(), "sk-ant-saisie-AB12");
}

#[test]
fn prod_refuse_une_cle_vide() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Prod, &store, env_empty);

    assert!(matches!(access.save("   "), Err(AppError::Validation(_))));
    assert_eq!(store.value().as_deref(), Some(STORED_KEY));
}

#[test]
fn prod_supprime_la_cle() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Prod, &store, env_empty);

    let status = access.delete().unwrap();
    assert_eq!(status, ApiKeyStatus { present: false, source: ApiKeySource::Keyring, last4: None });
    assert_eq!(store.value(), None);
    assert!(!access.status().unwrap().present);
}

#[test]
fn dev_refuse_l_enregistrement_et_la_suppression() {
    let store = Arc::new(FakeSecretStore::with(STORED_KEY));
    let access = access(RunMode::Dev, &store, env_with_key);

    match access.save("sk-ant-autre") {
        Err(AppError::Validation(message)) => assert!(message.contains(".env"), "{message}"),
        other => panic!("AppError::Validation attendue, reçu {other:?}"),
    }
    assert!(matches!(access.delete(), Err(AppError::Validation(_))));
    assert_eq!(store.accesses(), 0);
    assert_eq!(store.value().as_deref(), Some(STORED_KEY));
}

// ======================================================================
// Masquage et erreurs
// ======================================================================

#[test]
fn le_masquage_garde_les_4_derniers_caracteres() {
    assert_eq!(ApiKey::parse("sk-ant-api03-abcdWXYZ").unwrap().last4(), "WXYZ");
    assert_eq!(ApiKey::parse("abc").unwrap().last4(), "abc");
    assert_eq!(ApiKey::parse("  clé-éèàù  ").unwrap().last4(), "éèàù");
    assert!(ApiKey::parse(" \t ").is_none());
}

#[test]
fn le_debug_d_une_cle_ne_la_revele_pas() {
    let key = ApiKey::parse(STORED_KEY).unwrap();
    assert!(!format!("{key:?}").contains("KR42"));
}

#[test]
fn trousseau_indisponible_donne_une_erreur_francaise_explicite() {
    let cause = std::io::Error::other("org.freedesktop.secrets introuvable");
    let error = map_keyring_error(&KeyringError::NoStorageAccess(Box::new(cause)), KeyringAction::Read);
    match error {
        AppError::Config(message) => {
            assert!(message.starts_with("Trousseau du système indisponible"), "{message}");
            assert!(message.contains("impossible de lire la clé API Anthropic"), "{message}");
        }
        other => panic!("AppError::Config attendue, reçu {other:?}"),
    }
}
