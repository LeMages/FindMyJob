//! Endpoints REST pour les requêtes de recherche d'une session et le
//! déclenchement manuel de la recherche.

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::db;
use crate::scheduler;
use findmyjob_core::models::{CreateSearchQueryInput, SearchQueryView};

/// Assemble les routes `/api/sessions/{id}/search-queries*` et
/// `/api/sessions/{id}/search`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/sessions/{session_id}/search-queries",
            get(list_search_queries).post(create_search_query),
        )
        .route("/sessions/{session_id}/search-queries/{id}", delete(delete_search_query))
        .route("/sessions/{session_id}/search", post(run_session_search))
}

async fn list_search_queries(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<SearchQueryView>>, ApiError> {
    Ok(Json(db::search_queries::list_for_session(&state.pool, session_id).await?))
}

async fn create_search_query(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    Json(input): Json<CreateSearchQueryInput>,
) -> Result<Json<SearchQueryView>, ApiError> {
    Ok(Json(db::search_queries::insert(&state.pool, session_id, &input).await?))
}

async fn delete_search_query(
    State(state): State<AppState>,
    Path((session_id, id)): Path<(i64, i64)>,
) -> Result<(), ApiError> {
    db::search_queries::delete(&state.pool, session_id, id).await?;
    Ok(())
}

/// Déclenche une recherche complète (scraping + filtre + scoring) pour la
/// session, de façon synchrone — le bouton "Rechercher" du frontend attend
/// la fin de l'appel pour lever son état de chargement.
async fn run_session_search(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<(), ApiError> {
    scheduler::run_search_for_session(&state.pool, session_id, &state.config).await;
    Ok(())
}
