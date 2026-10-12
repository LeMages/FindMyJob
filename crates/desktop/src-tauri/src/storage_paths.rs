//! Construction et résolution de tous les chemins du dossier de données.
//!
//! En base, un chemin est toujours relatif au dossier de données, avec `/`
//! comme séparateur (`documents/sessions/1/cv/versions/4/reference.typ`) :
//! il ne dépend ni du système ni de l'emplacement du dossier. Ce module est
//! le seul à fabriquer ces chemins et à les convertir en chemins absolus.
//!
//! Arborescence :
//!
//! ```text
//! <dossier de données>/
//!   findmyjob.db
//!   polices/<famille>/*.ttf|otf
//!   documents/sessions/<session_id>/
//!     <doc_type>/versions/<version_id>/reference.typ, reference.pdf, template.typ, …
//!     <doc_type>/artefact/<candidature_id>.pdf, .data.json
//!     debug/<candidature_id>/*.json
//! ```

// Importation des modules standard
use std::path::{Component, Path, PathBuf};

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Nom du fichier de base de données.
const DB_FILE: &str = "findmyjob.db";

/// Nom du dossier des polices importées.
const FONTS_DIR: &str = "polices";

/// Chemin du fichier de base de données.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
///
/// # Returns
///
/// `<data_dir>/findmyjob.db`.
pub fn db_file(data_dir: &Path) -> PathBuf {
    data_dir.join(DB_FILE)
}

/// Dossier des polices importées par l'utilisateur.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
///
/// # Returns
///
/// `<data_dir>/polices`.
pub fn fonts_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(FONTS_DIR)
}

/// Dossier relatif des documents d'une session.
///
/// # Arguments
///
/// * `session_id` - Identifiant de la session.
///
/// # Returns
///
/// `documents/sessions/<session_id>`.
pub fn session_dir(session_id: i64) -> String {
    format!("documents/sessions/{session_id}")
}

/// Dossier relatif d'un type de document d'une session.
///
/// # Arguments
///
/// * `session_id` - Identifiant de la session.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"` (nom exact du dossier).
///
/// # Returns
///
/// `documents/sessions/<session_id>/<doc_type>`.
pub fn doc_type_dir(session_id: i64, doc_type: &str) -> String {
    format!("{}/{doc_type}", session_dir(session_id))
}

/// Dossier relatif d'une version de référence.
///
/// # Arguments
///
/// * `session_id` - Identifiant de la session.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version_id` - Identifiant de la version (`document_versions.id`).
///
/// # Returns
///
/// `documents/sessions/<session_id>/<doc_type>/versions/<version_id>`.
pub fn version_dir(session_id: i64, doc_type: &str, version_id: i64) -> String {
    format!("{}/versions/{version_id}", doc_type_dir(session_id, doc_type))
}

/// Fichier relatif d'un document généré pour une candidature.
///
/// # Arguments
///
/// * `session_id` - Identifiant de la session.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `candidature_id` - Identifiant de la candidature.
/// * `extension` - Extension sans point (`"pdf"`, `"data.json"`…).
///
/// # Returns
///
/// `documents/sessions/<session_id>/<doc_type>/artefact/<candidature_id>.<extension>`.
pub fn artefact_file(session_id: i64, doc_type: &str, candidature_id: i64, extension: &str) -> String {
    format!("{}/artefact/{candidature_id}.{extension}", doc_type_dir(session_id, doc_type))
}

/// Dossier relatif de la trace d'une génération.
///
/// # Arguments
///
/// * `session_id` - Identifiant de la session.
/// * `candidature_id` - Identifiant de la candidature.
///
/// # Returns
///
/// `documents/sessions/<session_id>/debug/<candidature_id>`.
pub fn debug_dir(session_id: i64, candidature_id: i64) -> String {
    format!("{}/debug/{candidature_id}", session_dir(session_id))
}

/// Chemin relatif d'un fichier dans un dossier relatif.
///
/// # Arguments
///
/// * `dir` - Dossier relatif (produit par ce module).
/// * `file_name` - Nom du fichier, sans séparateur.
///
/// # Returns
///
/// `<dir>/<file_name>`.
pub fn file_in(dir: &str, file_name: &str) -> String {
    format!("{dir}/{file_name}")
}

/// Convertit en chemin absolu un chemin relatif produit par l'application
/// (lu en base ou construit par ce module).
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `relative` - Chemin relatif, séparateur `/`.
///
/// # Returns
///
/// Chemin absolu, avec les séparateurs du système.
pub fn to_absolute(data_dir: &Path, relative: &str) -> PathBuf {
    relative.split('/').filter(|part| !part.is_empty()).fold(data_dir.to_path_buf(), |path, part| path.join(part))
}

/// Résout un chemin relatif reçu du frontend, en garantissant qu'il désigne
/// un emplacement sous le dossier de données : chemin absolu, préfixe de
/// lecteur et composant `..` sont refusés, et un fichier existant est
/// canonicalisé (liens symboliques compris) avant le contrôle d'appartenance.
///
/// # Arguments
///
/// * `data_dir` - Dossier de données de l'application.
/// * `relative` - Chemin relatif reçu (ex. `candidatures.cv_path` renvoyé par l'interface).
///
/// # Returns
///
/// Chemin absolu sous le dossier de données (le fichier peut ne pas
/// exister), ou `AppError::Validation` si le chemin en sort.
pub fn resolve_untrusted(data_dir: &Path, relative: &str) -> Result<PathBuf, AppError> {
    let refused = || AppError::Validation(format!("Chemin de document refusé : {relative}"));

    // ------------------------------------------------------------------
    // 1. Contrôle lexical : uniquement des composants normaux
    // ------------------------------------------------------------------
    let candidate = Path::new(relative);
    if relative.trim().is_empty() || !candidate.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(refused());
    }
    let absolute = to_absolute(data_dir, relative);

    // ------------------------------------------------------------------
    // 2. Fichier existant : chemin réel, contrôlé sous le dossier réel
    // ------------------------------------------------------------------
    match std::fs::canonicalize(&absolute) {
        Ok(real) => {
            let root = std::fs::canonicalize(data_dir).map_err(|e| {
                AppError::Config(format!("Dossier de données introuvable ({}) : {e}", data_dir.display()))
            })?;
            if real.starts_with(&root) { Ok(real) } else { Err(refused()) }
        }
        Err(_) => Ok(absolute),
    }
}
