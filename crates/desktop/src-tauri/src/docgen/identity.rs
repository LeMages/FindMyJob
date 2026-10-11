//! Identité du candidat (`Identity`) construite depuis le profil, commune
//! au CV et à la lettre.

// Importation des modules internes
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

/// Construit l'identité du candidat depuis le profil.
///
/// # Arguments
///
/// * `user` - Utilisateur du profil.
///
/// # Returns
///
/// `Identity`, champs absents en chaîne vide.
pub fn build_identity(user: &UserView) -> Identity {
    Identity {
        prenom: text(user.prenom.as_ref()),
        nom: text(user.nom.as_ref()),
        email: text(user.email.as_ref()),
        telephone: text(user.telephone.as_ref()),
        adresse: text(user.adresse.as_ref()),
        linkedin: display_url(user.linkedin.as_ref()),
        github: display_url(user.github.as_ref()),
        portfolio: display_url(user.portfolio.as_ref()),
        permis: user.permis.unwrap_or(false),
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
