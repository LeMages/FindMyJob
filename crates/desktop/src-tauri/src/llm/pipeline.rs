//! Résolution de l'AppHandle vers le dossier des polices importées, pour
//! déléguer la génération de documents au pipeline pur de
//! `findmyjob_core::llm::pipeline`.

// Importation des crates externes
use sqlx::SqlitePool;

// Importation des modules internes
use crate::error::AppError;
use findmyjob_core::llm::pipeline::GeneratedDocuments;

/// Génère le CV et la lettre de motivation adaptés à une offre.
///
/// # Arguments
///
/// * `app` - Handle de l'application Tauri (résolution du dossier des polices importées).
/// * `pool` - Pool de connexions SQLite.
/// * `session_id` - Identifiant de la session (fournit les documents de référence).
/// * `description_offre` - Texte brut de l'offre d'emploi.
///
/// # Returns
///
/// `GeneratedDocuments` avec le Typst et le PDF de chaque document.
pub async fn generate_documents(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    session_id: i64,
    description_offre: &str,
) -> Result<GeneratedDocuments, AppError> {
    let fonts_dir = crate::documents::fonts::resolve_fonts_dir(app);
    findmyjob_core::llm::pipeline::generate_documents(pool, &fonts_dir, session_id, description_offre).await
}
