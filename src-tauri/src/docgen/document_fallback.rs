//! Repli sur le document de référence, pour l'identité seulement : le
//! profil remplace une information du document dès qu'il la connaît ;
//! quand il est vide sur cette information, la valeur du document (données
//! d'exemple) est gardée. Les listes (compétences, langues, intérêts…) ne
//! se replient jamais sur la référence : le profil est leur seule source.

/// Valeur du profil si elle n'est pas vide, sinon celle du document.
///
/// # Arguments
///
/// * `profile` - Valeur tirée du profil (déjà nettoyée).
/// * `document` - Valeur des données d'exemple.
///
/// # Returns
///
/// Valeur à afficher.
pub fn text_or_document(profile: String, document: &str) -> String {
    if profile.trim().is_empty() {
        document.trim().to_string()
    } else {
        profile
    }
}
