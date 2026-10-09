//! Serveur HTTP Axum de l'agent : état applicatif, assemblage du routeur,
//! middleware d'authentification et CORS.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use axum::middleware;
use axum::Router;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

// Importation des modules internes
use crate::progress::ProgressStore;

mod auth;
mod candidatures;
mod common;
mod contacts;
mod document_versions;
mod documents;
mod error;
mod experiences;
mod fonts;
mod formations;
mod interviews;
mod langues;
mod profile;
mod projets;
mod scraping;
mod sessions;
mod settings;
mod skills;
mod status_history;
mod system;
mod timeline_events;
mod users;

/// État applicatif partagé, injecté dans chaque handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub data_dir: PathBuf,
    pub fonts_dir: PathBuf,
    pub progress: ProgressStore,
}

/// Construit la couche CORS : permissive si `CORS_ALLOWED_ORIGINS` n'est pas
/// configurée (mode développement), restreinte à la liste d'origines fournie
/// (séparées par des virgules) sinon.
fn build_cors_layer() -> CorsLayer {
    match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins) if !origins.trim().is_empty() => {
            let allowed: Vec<_> = origins
                .split(',')
                .filter_map(|o| o.trim().parse::<axum::http::HeaderValue>().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(allowed)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        }
        _ => CorsLayer::permissive(),
    }
}

/// Assemble le routeur HTTP complet de l'API, sous le préfixe `/api`, avec
/// authentification par jeton et CORS déjà appliqués.
///
/// # Arguments
///
/// * `state` - État applicatif partagé (pool DB, dossiers de données).
///
/// # Returns
///
/// Le `Router` Axum prêt à être servi.
pub fn router(state: AppState) -> Router {
    let authenticated = Router::new()
        .merge(users::router())
        .merge(profile::router())
        .merge(experiences::router())
        .merge(formations::router())
        .merge(langues::router())
        .merge(projets::router())
        .merge(skills::router())
        .merge(sessions::router())
        .merge(candidatures::router())
        .merge(contacts::router())
        .merge(status_history::router())
        .merge(timeline_events::router())
        .merge(interviews::router())
        .merge(scraping::router())
        .merge(documents::router())
        .merge(document_versions::router())
        .merge(fonts::router())
        .merge(settings::router())
        .route_layer(middleware::from_fn(auth::require_bearer_token));

    // `/api/system/health` reste volontairement hors de l'authentification
    // par jeton : une sonde de supervision doit pouvoir vérifier la
    // disponibilité du serveur sans en connaître le jeton.
    let api = Router::new().merge(system::health_router()).merge(authenticated);

    Router::new().nest("/api", api).layer(build_cors_layer()).with_state(state)
}
