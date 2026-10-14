//! Identité du candidat (`Identity`) construite depuis le profil, commune
//! au CV et à la lettre.

// Importation des modules internes
use crate::docgen::document_fallback::text_or_document;
use findmyjob_core::models::{Identity, UserView};

/// Valeur d'un champ optionnel du profil, nettoyée (chaîne vide si absent).
pub fn text(value: Option<&String>) -> String {
    value.map(|v| v.trim().to_string()).unwrap_or_default()
}

/// Forme affichable d'un lien : sans protocole, sans `www.` ni barre finale,
/// comme on l'écrit sur un CV (« linkedin.com/in/… »).
///
/// # Arguments
///
/// * `url` - Lien tel que saisi dans le profil.
///
/// # Returns
///
/// Lien affichable, chaîne vide si absent.
pub fn display_url(url: Option<&String>) -> String {
    let raw = text(url);
    let without_scheme = raw.strip_prefix("https://").or_else(|| raw.strip_prefix("http://")).unwrap_or(&raw);
    let without_www = without_scheme.strip_prefix("www.").unwrap_or(without_scheme);
    without_www.trim_end_matches('/').to_string()
}

/// Construit l'identité du candidat : chaque champ vient du profil, ou du
/// document de référence quand le profil est vide sur ce champ.
///
/// # Arguments
///
/// * `user` - Utilisateur du profil.
/// * `title` - Sous-titre professionnel (rédigé avec le CV), commun aux
///   deux documents.
/// * `document` - Identité des données d'exemple de la référence.
///
/// # Returns
///
/// `Identity`, chaîne vide si ni le profil ni le document n'ont la valeur.
pub fn build_identity(user: &UserView, title: &str, document: &Identity) -> Identity {
    Identity {
        prenom: text_or_document(text(user.prenom.as_ref()), &document.prenom),
        nom: text_or_document(text(user.nom.as_ref()), &document.nom),
        title: text_or_document(title.trim().to_string(), &document.title),
        email: text_or_document(text(user.email.as_ref()), &document.email),
        telephone: text_or_document(text(user.telephone.as_ref()), &document.telephone),
        ville: text_or_document(text(user.ville.as_ref()), &document.ville),
        zip_code: text_or_document(text(user.zip_code.as_ref()), &document.zip_code),
        linkedin: text_or_document(display_url(user.linkedin.as_ref()), &document.linkedin),
        github: text_or_document(display_url(user.github.as_ref()), &document.github),
        portfolio: text_or_document(display_url(user.portfolio.as_ref()), &document.portfolio),
        permis: user.permis.unwrap_or(document.permis),
    }
}

/// Découpe un texte multiligne du profil en éléments (une ligne = un
/// élément), sans les puces éventuellement saisies en tête de ligne.
///
/// # Arguments
///
/// * `value` - Texte du profil (détails de formation, centres d'intérêt).
///
/// # Returns
///
/// Éléments non vides, dans l'ordre.
pub fn lines(value: Option<&String>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    value
        .lines()
        .map(|line| line.trim().trim_start_matches(['•', '-', '*', '–', '—']).trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
