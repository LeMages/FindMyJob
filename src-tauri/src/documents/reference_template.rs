//! Chargement d'une référence validée (template et contrat enregistré)
//! pour la génération d'une candidature.
//!
//! Une version importée avant le contrat FindMyJob (sans `contract.json`,
//! ou avec un autre `template.typ` que sa source) ou d'une autre version du
//! contrat doit être réimportée : `AppError::OutdatedReference`. Un
//! `contract.json` présent mais illisible est un fichier corrompu, pas une
//! référence obsolète : `AppError::Validation`.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;

// Importation des modules internes
use crate::db;
use crate::error::AppError;
use crate::models::{StoredContract, TemplateArtifacts, CONTRACT_FILE, CONTRACT_VERSION};
use crate::storage_paths;

/// Référence prête pour la génération.
///
/// # Attributes
///
/// * `template` - Source Typst de la référence (le template lui-même).
/// * `contract` - Contrat enregistré à l'import, exemple typé.
/// * `page_count` - Nombre de pages maximal des documents générés.
/// * `version_id` - Version de référence utilisée (`document_versions.id`).
pub struct ReferenceTemplate<E> {
    pub template: String,
    pub contract: StoredContract<E>,
    pub page_count: usize,
    pub version_id: i64,
}

/// Charge la référence d'un type de document pour une session.
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
/// `ReferenceTemplate`, `AppError::Validation` si la référence est absente
/// ou son contrat corrompu, `AppError::OutdatedReference` si elle doit être
/// réimportée.
pub async fn load_reference<E: DeserializeOwned>(
    pool: &SqlitePool,
    data_dir: &Path,
    session_id: i64,
    doc_type: &str,
    label: &str,
) -> Result<ReferenceTemplate<E>, AppError> {
    let doc = db::documents::get_current(pool, session_id, doc_type).await?;
    let (doc, artifacts) = doc.and_then(|d| d.template().map(|t| (d, t))).ok_or_else(|| {
        AppError::Validation(format!(
            "Veuillez d'abord importer votre {label} de référence dans les paramètres de session."
        ))
    })?;
    if !is_contract_version(&artifacts, &doc.file_path) {
        return Err(outdated(label));
    }

    let contract = load_contract(&storage_paths::to_absolute(data_dir, &artifacts.example_data_path), label)?;
    let template = read(&storage_paths::to_absolute(data_dir, &artifacts.template_path))?;
    let page_count = usize::try_from(artifacts.page_count)
        .map_err(|_| AppError::Validation(format!("Nombre de pages invalide : {}", artifacts.page_count)))?;

    Ok(ReferenceTemplate { template, contract, page_count, version_id: doc.current_version_id })
}

/// Vrai si une version a été importée par le contrat : son template est sa
/// source et ses données sont un `contract.json`.
fn is_contract_version(artifacts: &TemplateArtifacts, typst_path: &str) -> bool {
    artifacts.template_path == typst_path && artifacts.example_data_path.ends_with(CONTRACT_FILE)
}

/// Lit le contrat enregistré d'une version.
///
/// # Arguments
///
/// * `path` - Chemin absolu de `contract.json`.
/// * `label` - Libellé lisible du document, pour les messages.
///
/// # Returns
///
/// `StoredContract`, `AppError::OutdatedReference` s'il est absent ou d'une
/// autre version du contrat, `AppError::Validation` s'il est illisible.
pub fn load_contract<E: DeserializeOwned>(path: &Path, label: &str) -> Result<StoredContract<E>, AppError> {
    // 1. Fichier absent : version importée avant le contrat
    let Ok(content) = std::fs::read_to_string(path) else {
        return Err(outdated(label));
    };
    // 2. Version du contrat lue seule d'abord : un autre schéma ne doit pas
    //    passer pour un fichier corrompu.
    let version = serde_json::from_str::<serde_json::Value>(&content)
        .ok()
        .and_then(|value| value.get("contract_version").and_then(serde_json::Value::as_u64));
    match version {
        Some(v) if v == u64::from(CONTRACT_VERSION) => {}
        Some(_) => return Err(outdated(label)),
        None => return Err(corrupted(path, label, "version du contrat absente")),
    }
    serde_json::from_str(&content).map_err(|e| corrupted(path, label, &e.to_string()))
}

/// Erreur d'une référence importée par une version précédente.
fn outdated(label: &str) -> AppError {
    AppError::OutdatedReference(format!(
        "Votre {label} de référence a été importé avec une version précédente de l'application et n'est plus \
         compatible. Réimportez-le dans les paramètres de session."
    ))
}

/// Erreur d'un contrat enregistré illisible.
fn corrupted(path: &Path, label: &str, detail: &str) -> AppError {
    AppError::Validation(format!(
        "Le fichier de votre {label} de référence est corrompu ({}) : {detail}. Réimportez-le dans les paramètres \
         de session.",
        path.display()
    ))
}

/// Lit un fichier de la référence.
fn read(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire {} : {e}", path.display())))
}
