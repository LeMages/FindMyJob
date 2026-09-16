//! Endpoints REST pour les paramètres applicatifs : clé API Anthropic,
//! modèle Claude sélectionné, thème, test de connexion.

// Importation des crates externes
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::{db, security};
use findmyjob_core::error::AppError;
use findmyjob_core::llm::client::{call_claude_with_tokens, AVAILABLE_MODELS, DEFAULT_MODEL};

/// Statut de la clé API, sans jamais exposer sa valeur réelle.
#[derive(serde::Serialize)]
struct ApiKeyStatus {
    exists: bool,
    source: &'static str,
}

/// Résultat du test de connexion à l'API Claude.
#[derive(serde::Serialize)]
struct TestConnectionResult {
    success: bool,
    model: String,
    message: String,
    latency_ms: u64,
}

#[derive(serde::Deserialize)]
struct ApiKeyInput {
    key: String,
}

#[derive(serde::Deserialize)]
struct ModelInput {
    model: String,
}

#[derive(serde::Deserialize)]
struct ThemeInput {
    theme: String,
}

/// Assemble les routes `/api/settings/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings/api-key", get(get_api_key_status).put(set_api_key).delete(delete_api_key))
        .route("/settings/model", get(get_llm_model).put(set_llm_model))
        .route("/settings/theme", get(get_theme).put(set_theme))
        .route("/settings/test-connection", post(test_api_connection))
}

async fn get_api_key_status() -> Json<ApiKeyStatus> {
    Json(ApiKeyStatus { exists: security::has_api_key(), source: security::key_source() })
}

async fn set_api_key(Json(input): Json<ApiKeyInput>) -> Result<(), ApiError> {
    let key = input.key.trim();
    if key.is_empty() {
        return Err(AppError::Validation("La clé API ne peut pas être vide.".to_string()).into());
    }
    security::set_api_key(key);
    Ok(())
}

async fn delete_api_key() -> Result<(), ApiError> {
    security::delete_api_key();
    Ok(())
}

async fn get_llm_model(State(state): State<AppState>) -> Result<Json<String>, ApiError> {
    Ok(Json(db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?))
}

async fn set_llm_model(
    State(state): State<AppState>,
    Json(input): Json<ModelInput>,
) -> Result<(), ApiError> {
    if !AVAILABLE_MODELS.contains(&input.model.as_str()) {
        return Err(AppError::Validation("Modèle Claude invalide.".to_string()).into());
    }
    db::settings::set(&state.pool, db::settings::KEY_LLM_MODEL, &input.model).await?;
    Ok(())
}

async fn get_theme(State(state): State<AppState>) -> Result<Json<String>, ApiError> {
    Ok(Json(db::settings::resolve_theme(&state.pool).await?))
}

async fn set_theme(State(state): State<AppState>, Json(input): Json<ThemeInput>) -> Result<(), ApiError> {
    if !db::settings::VALID_THEMES.contains(&input.theme.as_str()) {
        return Err(AppError::Validation("Thème invalide.".to_string()).into());
    }
    db::settings::set(&state.pool, db::settings::KEY_THEME, &input.theme).await?;
    Ok(())
}

async fn test_api_connection(State(state): State<AppState>) -> Result<Json<TestConnectionResult>, ApiError> {
    let model = db::settings::resolve_llm_model(&state.pool, DEFAULT_MODEL).await?;

    let started = std::time::Instant::now();
    let result = call_claude_with_tokens(
        "Réponds uniquement par le mot OK.",
        "Test de connexion.",
        None,
        10,
        &model,
    )
    .await;
    let latency_ms = started.elapsed().as_millis() as u64;

    let (success, message) = match result {
        Ok(_) => (true, "Connexion réussie.".to_string()),
        Err(e) => (false, e.to_string()),
    };

    Ok(Json(TestConnectionResult { success, model, message, latency_ms }))
}
