//! Repli sur le document de référence : le profil remplace une information
//! du document dès qu'il la connaît ; quand il est vide sur cette
//! information, la valeur du document (données d'exemple) est gardée. Le
//! profil sert à suivre une évolution en cours de recherche, le document
//! est en général déjà à jour.

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

/// Entrées du profil s'il en a, sinon celles du document.
///
/// # Arguments
///
/// * `profile` - Entrées construites depuis le profil.
/// * `document` - Entrées des données d'exemple.
///
/// # Returns
///
/// Entrées à afficher.
pub fn list_or_document<T: Clone>(profile: Vec<T>, document: &[T]) -> Vec<T> {
    if profile.is_empty() {
        document.to_vec()
    } else {
        profile
    }
}
