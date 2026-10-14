//! Validation du type de document de référence reçu du frontend.

// Importation des modules internes
use findmyjob_core::error::AppError;

/// Vérifie qu'un type de document reçu est connu.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`, tel qu'envoyé par l'interface.
///
/// # Returns
///
/// Le type sous forme statique, ou `AppError::Validation` pour toute autre valeur.
pub fn parse_doc_type(doc_type: &str) -> Result<&'static str, AppError> {
    match doc_type {
        "cv" => Ok("cv"),
        "lettre_motivation" => Ok("lettre_motivation"),
        other => Err(AppError::Validation(format!("Type de document inconnu : « {other} »."))),
    }
}
