//! Endpoints REST pour les polices importées pour la compilation Typst.
//!
//! `import_font_folder` prend un chemin de dossier source sur le
//! **filesystem du serveur** (ex. un dossier monté dans `DATA_DIR`) — comme
//! côté desktop, il n'y a pas d'upload de fichiers ici (hors scope de cette étape).

// Importation des crates externes
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};

// Importation des modules internes
use crate::api::error::ApiError;
use crate::api::AppState;
use crate::documents;
use findmyjob_core::models::FontFamilyView;

#[derive(serde::Deserialize)]
struct ImportFontInput {
    source_path: String,
}

/// Assemble les routes `/api/fonts*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/fonts", get(list_fonts).post(import_font_folder))
        .route("/fonts/{name}", delete(delete_font))
}

async fn list_fonts(State(state): State<AppState>) -> Result<Json<Vec<FontFamilyView>>, ApiError> {
    Ok(Json(documents::list_fonts(&state.fonts_dir)?))
}

async fn import_font_folder(
    State(state): State<AppState>,
    Json(input): Json<ImportFontInput>,
) -> Result<Json<FontFamilyView>, ApiError> {
    Ok(Json(documents::import_font_folder(&state.fonts_dir, &input.source_path)?))
}

async fn delete_font(State(state): State<AppState>, Path(name): Path<String>) -> Result<(), ApiError> {
    documents::delete_font(&state.fonts_dir, &name)?;
    Ok(())
}
