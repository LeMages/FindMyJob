//! Middleware d'authentification : jeton statique porté en `Bearer`.
//!
//! Le jeton attendu vient de la variable d'environnement `API_TOKEN`, lue à
//! chaque requête (coût négligeable, évite un état supplémentaire dans
//! `AppState`). Si elle n'est pas configurée, toutes les requêtes passent
//! (mode développement) — un avertissement est affiché au démarrage par
//! `main.rs`.

// Importation des crates externes
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Vérifie le jeton `Bearer` porté par la requête, si `API_TOKEN` est configuré.
///
/// # Returns
///
/// La réponse du handler suivant si l'authentification passe (ou si aucun
/// jeton n'est configuré), sinon une réponse `401 Unauthorized`.
pub async fn require_bearer_token(request: Request, next: Next) -> Response {
    let Ok(expected) = std::env::var("API_TOKEN") else {
        return next.run(request).await;
    };
    if expected.is_empty() {
        return next.run(request).await;
    }

    let provided = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if provided != Some(expected.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "kind": "Unauthorized",
                "message": "Jeton d'authentification manquant ou invalide."
            })),
        )
            .into_response();
    }

    next.run(request).await
}
