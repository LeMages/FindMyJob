//! Règles des paramètres applicatifs : modèle Claude et thème acceptés,
//! test de connexion à l'API, gestion de la clé API.

// Importation des modules standard
use std::time::Instant;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::app_state::AppState;
use crate::db;
use crate::services::api_key::ApiKeyAccess;
use crate::error::AppError;
use crate::llm::client::{call_claude_with_tokens, AVAILABLE_MODELS, DEFAULT_MODEL};
use crate::models::{ApiKeyStatus, TestConnectionResult};

/// Enregistre le modèle Claude choisi, s'il fait partie des modèles proposés.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `model` - Identifiant du modèle.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` pour un modèle inconnu.
pub async fn set_llm_model(pool: &SqlitePool, model: &str) -> Result<(), AppError> {
    if !AVAILABLE_MODELS.contains(&model) {
        return Err(AppError::Validation("Modèle Claude invalide.".to_string()));
    }
    db::settings::set(pool, db::settings::KEY_LLM_MODEL, model).await
}

/// Enregistre le thème d'affichage choisi, s'il est valide.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `theme` - `"clair"`, `"sombre"` ou `"systeme"`.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` pour un thème inconnu.
pub async fn set_theme(pool: &SqlitePool, theme: &str) -> Result<(), AppError> {
    if !db::settings::VALID_THEMES.contains(&theme) {
        return Err(AppError::Validation("Thème invalide.".to_string()));
    }
    db::settings::set(pool, db::settings::KEY_THEME, theme).await
}

/// Teste la connexion à l'API Claude avec le modèle configuré, en mesurant
/// la latence. Un échec d'appel n'est pas une erreur de la commande : il est
/// rapporté dans le résultat.
///
/// # Arguments
///
/// * `state` - État de l'application (modèle configuré, clé API).
///
/// # Returns
///
/// `TestConnectionResult` (succès, modèle, message, latence).
pub async fn test_connection(state: &AppState) -> Result<TestConnectionResult, AppError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;
    // Une clé absente ou un trousseau inaccessible est un échec du test,
    // rapporté dans le résultat comme un échec d'appel.
    let target = match state.claude().resolve().await {
        Ok(target) => target,
        Err(e) => return Ok(TestConnectionResult { success: false, model, message: e.to_string(), latency_ms: 0 }),
    };

    let started = Instant::now();
    let result = call_claude_with_tokens("test de connexion", "Réponds uniquement par le mot OK.", "Test de connexion.", None, 10, &target).await;
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    let (success, message) = match result {
        Ok(_) => (true, "Connexion réussie.".to_string()),
        Err(e) => (false, e.to_string()),
    };
    Ok(TestConnectionResult { success, model: target.model, message, latency_ms })
}

/// État de la clé API Anthropic (présence, source, 4 derniers caractères).
///
/// # Arguments
///
/// * `access` - Accès à la clé selon le mode d'exécution.
///
/// # Returns
///
/// `ApiKeyStatus`, ou `AppError::Config` si le trousseau est inaccessible.
pub async fn api_key_status(access: &ApiKeyAccess) -> Result<ApiKeyStatus, AppError> {
    run_blocking(access, |access| access.status()).await
}

/// Enregistre la clé API Anthropic dans le trousseau (mode PROD uniquement).
///
/// # Arguments
///
/// * `access` - Accès à la clé selon le mode d'exécution.
/// * `key` - Clé saisie.
///
/// # Returns
///
/// L'état de la clé enregistrée, ou l'erreur de validation ou du trousseau.
pub async fn save_api_key(access: &ApiKeyAccess, key: String) -> Result<ApiKeyStatus, AppError> {
    run_blocking(access, move |access| access.save(&key)).await
}

/// Supprime la clé API Anthropic du trousseau (mode PROD uniquement).
///
/// # Arguments
///
/// * `access` - Accès à la clé selon le mode d'exécution.
///
/// # Returns
///
/// L'état de la clé (absente), ou l'erreur de validation ou du trousseau.
pub async fn delete_api_key(access: &ApiKeyAccess) -> Result<ApiKeyStatus, AppError> {
    run_blocking(access, |access| access.delete()).await
}

/// Exécute une opération sur la clé hors du runtime asynchrone : le
/// trousseau du système est une API bloquante (D-Bus sous Linux).
async fn run_blocking<T, F>(access: &ApiKeyAccess, operation: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(&ApiKeyAccess) -> Result<T, AppError> + Send + 'static,
{
    let access = access.clone();
    tokio::task::spawn_blocking(move || operation(&access))
        .await
        .map_err(|e| AppError::Config(format!("Opération sur la clé API interrompue : {e}")))?
}
