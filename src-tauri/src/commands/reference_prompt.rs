//! Commandes Tauri de l'aide à la préparation d'une référence : prompt de
//! conversion au contrat FindMyJob (bouton « Copier le prompt pour
//! Claude ») et liens vers le contrat et les exemples.

// Importation des modules internes
use crate::error::AppError;
use crate::models::reference_doc_links::ReferenceDocLinks;
use crate::services::reference_prompt;

/// Renvoie le prompt de conversion d'un CV ou d'une lettre.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `lang` - Langue de l'interface (`"fr"` ou `"en"`).
///
/// # Returns
///
/// Le prompt complet, prêt à copier.
#[tauri::command]
pub fn get_reference_prompt(doc_type: String, lang: String) -> Result<String, AppError> {
    reference_prompt::reference_prompt(&doc_type, &lang)
}

/// Renvoie les liens du contrat et des exemples conformes.
///
/// # Arguments
///
/// * `lang` - Langue de l'interface (`"fr"` ou `"en"`).
///
/// # Returns
///
/// Les liens GitHub à afficher.
#[tauri::command]
pub fn get_reference_doc_links(lang: String) -> Result<ReferenceDocLinks, AppError> {
    reference_prompt::reference_doc_links(&lang)
}
