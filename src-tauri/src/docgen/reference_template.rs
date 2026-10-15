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
    let template = read(&template_file)?;
    let example_json = read(&storage_paths::to_absolute(data_dir, &artifacts.example_data_path))?;
    check_current_schema(&template, &example_json, label)?;
    let example = serde_json::from_str(&example_json).map_err(|e| {
        AppError::Validation(format!("Données d'exemple illisibles ({}) : {e}", artifacts.example_data_path))
    })?;
    let dir = template_file
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::Validation(format!("Chemin de template invalide : {}", artifacts.template_path)))?;
    let page_count = usize::try_from(artifacts.page_count)
        .map_err(|_| AppError::Validation(format!("Nombre de pages invalide : {}", artifacts.page_count)))?;

    Ok(ReferenceTemplate { template, example, dir, page_count, version_id: doc.current_version_id })
}

/// Champs retirés du schéma canonique : un template qui les lit a été
/// converti par une version précédente et ne compilerait plus.
const REMOVED_FIELDS: [&str; 1] = ["identity.adresse"];
/// Clés ajoutées aux données d'exemple : une référence convertie avant leur
/// ajout n'a ni consignes de rédaction, ni texte sans champ gardé en dur.
const REQUIRED_EXAMPLE_KEYS: [&str; 1] = ["guidance"];

/// Refuse une référence convertie par une version précédente : template
/// qui lit un champ retiré du schéma (`REMOVED_FIELDS`), ou données
/// d'exemple sans une clé ajoutée depuis (`REQUIRED_EXAMPLE_KEYS`). L'erreur
/// demande de réimporter la référence plutôt qu'une erreur de compilation
/// Typst ou de lecture.
///
/// # Arguments
///
/// * `template` - Source du `template.typ`.
/// * `example_json` - Contenu de `example_data.json`.
/// * `label` - Libellé lisible du document, pour le message.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::OutdatedReference`.
pub fn check_current_schema(template: &str, example_json: &str, label: &str) -> Result<(), AppError> {
    // Un JSON illisible n'est pas une ancienne version : sa lecture typée,
    // juste après, le signale avec son erreur.
    let missing_key = match serde_json::from_str::<serde_json::Value>(example_json) {
        Ok(example) => REQUIRED_EXAMPLE_KEYS.iter().any(|key| example.get(key).is_none()),
        Err(_) => false,
    };
    if missing_key || REMOVED_FIELDS.iter().any(|field| template.contains(field)) {
        return Err(AppError::OutdatedReference(format!(
            "Votre {label} de référence a été converti par une version précédente de l'application et n'est \
             plus compatible. Réimportez-le dans les paramètres de session."
        )));
    }
    Ok(())
}

/// Lit un fichier de la référence.
fn read(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire {} : {e}", path.display())))
}
