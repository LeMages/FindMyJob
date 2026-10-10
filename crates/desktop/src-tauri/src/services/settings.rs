//! Règles des paramètres applicatifs : modèle Claude et thème acceptés,
//! test de connexion à l'API.

// Importation des modules standard
use std::time::Instant;

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use findmyjob_core::error::AppError;
use findmyjob_core::llm::client::{call_claude_with_tokens, AVAILABLE_MODELS, DEFAULT_MODEL};
use findmyjob_core::models::TestConnectionResult;

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
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// `TestConnectionResult` (succès, modèle, message, latence).
pub async fn test_connection(pool: &SqlitePool) -> Result<TestConnectionResult, AppError> {
    let model = db::settings::resolve_llm_model(pool, DEFAULT_MODEL).await?;

    let started = Instant::now();
    let result = call_claude_with_tokens("Réponds uniquement par le mot OK.", "Test de connexion.", None, 10, &model).await;
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    let (success, message) = match result {
        Ok(_) => (true, "Connexion réussie.".to_string()),
        Err(e) => (false, e.to_string()),
    };
    Ok(TestConnectionResult { success, model, message, latency_ms })
}
