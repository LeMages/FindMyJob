//! Chargement d'une référence templatisée (template, données d'exemple,
//! nombre de pages) pour la génération d'une candidature.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use serde::de::DeserializeOwned;
use sqlx::PgPool;

// Importation des modules internes
use crate::db;
use findmyjob_core::error::AppError;

/// Référence prête pour la génération.
///
/// # Attributes
///
/// * `template` - Source du `template.typ`.
/// * `example` - Contenu de `example_data.json`, typé.
/// * `dir` - Dossier de la version (racine de compilation, cache des libellés).
/// * `page_count` - Nombre de pages maximal des documents générés.
/// * `version_id` - Version de référence utilisée (`document_versions.id`).
pub struct ReferenceTemplate<E> {
    pub template: String,
    pub example: E,
    pub dir: PathBuf,
    pub page_count: usize,
    pub version_id: Option<i64>,
}

/// Charge la référence templatisée d'un type de document pour une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions PostgreSQL.
/// * `session_id` - Identifiant de la session.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `label` - Libellé lisible du document, pour les messages d'erreur.
///
/// # Returns
///
/// `ReferenceTemplate`, ou `AppError::Validation` si la référence est
/// absente ou pas encore templatisée.
pub async fn load_reference<E: DeserializeOwned>(
    pool: &PgPool,
    session_id: i64,
    doc_type: &str,
    label: &str,
) -> Result<ReferenceTemplate<E>, AppError> {
    let doc = db::documents::get_by_session_and_type(pool, session_id, doc_type).await?;
    let (doc, artifacts) = doc
        .and_then(|d| d.template().map(|t| (d, t)))
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Veuillez d'abord importer votre {label} de référence dans les paramètres de session."
            ))
        })?;

    let template = read(&artifacts.template_path)?;
    let example_json = read(&artifacts.example_data_path)?;
    let example = serde_json::from_str(&example_json).map_err(|e| {
        AppError::Validation(format!("Données d'exemple illisibles ({}) : {e}", artifacts.example_data_path))
    })?;
    let dir = Path::new(&artifacts.template_path)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::Validation(format!("Chemin de template invalide : {}", artifacts.template_path)))?;
    let page_count = usize::try_from(artifacts.page_count)
        .map_err(|_| AppError::Validation(format!("Nombre de pages invalide : {}", artifacts.page_count)))?;

    Ok(ReferenceTemplate { template, example, dir, page_count, version_id: doc.current_version_id })
}

/// Lit un fichier de la référence.
fn read(path: &str) -> Result<String, AppError> {
    std::fs::read_to_string(path).map_err(|e| AppError::Validation(format!("Impossible de lire {path} : {e}")))
}
