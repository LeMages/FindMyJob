//! Liens vers la documentation du contrat des références, affichés dans
//! l'aide à la préparation d'une référence.

// Importation des crates externes
use serde::Serialize;

/// Liens GitHub du contrat et des exemples conformes.
///
/// # Attributes
///
/// * `contract` - Contrat, dans la langue demandée.
/// * `cv_example` - Exemple de CV conforme.
/// * `lm_example` - Exemple de lettre conforme.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReferenceDocLinks {
    pub contract: String,
    pub cv_example: String,
    pub lm_example: String,
}
