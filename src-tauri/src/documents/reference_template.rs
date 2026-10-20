//! Chargement d'une référence templatisée (template, données d'exemple,
//! nombre de pages) pour la génération d'une candidature.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des crates externes
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::storage_paths;
use crate::error::AppError;
use crate::models::{ReferenceTexts, REFERENCE_FORMAT_VERSION, REFERENCE_TEXTS_FILE};

/// Référence prête pour la génération.
///
/// # Attributes
///
/// * `template` - Source du `template.typ`.
/// * `example` - Contenu de `example_data.json`, typé.
/// * `texts` - Textes en dur et consignes de la référence (`reference_texts.json`).
/// * `dir` - Dossier de la version (racine de compilation, cache des libellés).
/// * `page_count` - Nombre de pages maximal des documents générés.
/// * `version_id` - Version de référence utilisée (`document_versions.id`).
pub struct ReferenceTemplate<E> {
    pub template: String,
    pub example: E,
    pub texts: ReferenceTexts,
    pub dir: PathBuf,
    pub page_count: usize,
    pub version_id: i64,
}

/// Charge la référence templatisée d'un type de document pour une session.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `data_dir` - Dossier de données de l'application.
/// * `session_id` - Identifiant de la session.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `label` - Libellé lisible du document, pour les messages d'erreur.
///
/// # Returns
///
/// `ReferenceTemplate`, ou `AppError::Validation` si la référence est
/// absente ou pas encore templatisée.
pub async fn load_reference<E: DeserializeOwned>(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    label: &str,
) -> Result<ReferenceTemplate<E>, AppError> {
    let doc = db::documents::get_current(pool, session_id, doc_type).await?;
    let (doc, artifacts) = doc
        .and_then(|d| d.template().map(|t| (d, t)))
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Veuillez d'abord importer votre {label} de référence dans les paramètres de session."
            ))
        })?;

    let template_file = storage_paths::to_absolute(data_dir, &artifacts.template_path);
    let dir = template_file
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::Validation(format!("Chemin de template invalide : {}", artifacts.template_path)))?;
    let texts = load_texts(&dir.join(REFERENCE_TEXTS_FILE), label)?;
    let template = read(&template_file)?;
    let example_json = read(&storage_paths::to_absolute(data_dir, &artifacts.example_data_path))?;
    let example = serde_json::from_str(&example_json).map_err(|_| outdated(label))?;
    let page_count = usize::try_from(artifacts.page_count)
        .map_err(|_| AppError::Validation(format!("Nombre de pages invalide : {}", artifacts.page_count)))?;

    Ok(ReferenceTemplate { template, example, texts, dir, page_count, version_id: doc.current_version_id })
}

/// Charge les textes de la référence. Une version sans ce fichier, ou d'un
/// autre format, a été convertie par une version précédente de
/// l'application : elle doit être réimportée.
///
/// # Arguments
///
/// * `path` - Chemin de `reference_texts.json`.
/// * `label` - Libellé lisible du document, pour le message.
///
/// # Returns
///
/// `ReferenceTexts`, ou `AppError::OutdatedReference`.
pub fn load_texts(path: &Path, label: &str) -> Result<ReferenceTexts, AppError> {
    let content = std::fs::read_to_string(path).map_err(|_| outdated(label))?;
    let texts: ReferenceTexts = serde_json::from_str(&content).map_err(|_| outdated(label))?;
    if texts.format_version != REFERENCE_FORMAT_VERSION {
        return Err(outdated(label));
    }
    Ok(texts)
}

/// Erreur d'une référence convertie par une version précédente.
fn outdated(label: &str) -> AppError {
    AppError::OutdatedReference(format!(
        "Votre {label} de référence a été importé avec une version précédente de l'application et n'est plus          compatible. Réimportez-le dans les paramètres de session."
    ))
}

/// Lit un fichier de la référence.
fn read(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire {} : {e}", path.display())))
}
