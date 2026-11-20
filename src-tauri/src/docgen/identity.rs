//! Identité du candidat (`Identity`) construite depuis le profil, commune
//! au CV et à la lettre.

// Importation des modules internes
use crate::error::AppError;
use crate::models::{Identity, UserView};

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

/// Localisation de l'expéditeur, propre à la session. Elle ne se replie
/// jamais sur la référence : une ville absente reste vide.
///
/// # Attributes
///
/// * `ville` - Ville de la session (chaîne vide si absente).
/// * `zip_code` - Code postal de la session (chaîne vide si absent).
#[derive(Clone, Debug, Default)]
pub struct SenderLocation {
    pub ville: String,
    pub zip_code: String,
}

impl SenderLocation {
    /// Construit la localisation depuis les champs optionnels d'une session.
    ///
    /// # Arguments
    ///
    /// * `ville` - Ville de la session.
    /// * `zip_code` - Code postal de la session.
    ///
    /// # Returns
    ///
    /// Localisation nettoyée, chaînes vides si absente.
    pub fn new(ville: Option<&String>, zip_code: Option<&String>) -> Self {
        Self { ville: text(ville), zip_code: text(zip_code) }
    }
}

/// Construit l'identité du candidat : chaque champ vient du profil (le
/// sous-titre, de la rédaction du CV), le lieu de la session. Jamais de
/// repli sur la référence : un champ vide reste vide et sa ligne disparaît.
///
/// # Arguments
///
/// * `user` - Utilisateur du profil.
/// * `location` - Localisation de l'expéditeur (session).
/// * `title` - Sous-titre professionnel (rédigé avec le CV), commun aux
///   deux documents.
///
/// # Returns
///
/// `Identity`, chaîne vide quand la valeur est absente.
pub fn build_identity(user: &UserView, location: &SenderLocation, title: &str) -> Identity {
    Identity {
        prenom: text(user.prenom.as_ref()),
        nom: text(user.nom.as_ref()),
        title: title.trim().to_string(),
        email: text(user.email.as_ref()),
        telephone: text(user.telephone.as_ref()),
        ville: location.ville.clone(),
        zip_code: location.zip_code.clone(),
        linkedin: display_url(user.linkedin.as_ref()),
        github: display_url(user.github.as_ref()),
        portfolio: display_url(user.portfolio.as_ref()),
        // Permis non renseigné : ligne retirée (avertissement), jamais reprise de la référence.
        permis: user.permis.unwrap_or(false),
    }
}

/// Vérifie que le profil porte un prénom et un nom : ils signent la lettre
/// et ouvrent le CV, aucune valeur d'exemple ne peut les remplacer.
///
/// # Arguments
///
/// * `user` - Utilisateur du profil.
///
/// # Returns
///
/// `Ok(())`, ou `AppError::Validation` nommant le champ du profil à compléter.
pub fn validate_sender_name(user: &UserView) -> Result<(), AppError> {
    if text(user.prenom.as_ref()).is_empty() {
        return Err(AppError::Validation(
            "Le prénom du profil est vide : renseigne-le dans le profil avant de générer des documents.".to_string(),
        ));
    }
    if text(user.nom.as_ref()).is_empty() {
        return Err(AppError::Validation(
            "Le nom du profil est vide : renseigne-le dans le profil avant de générer des documents.".to_string(),
        ));
    }
    Ok(())
}
