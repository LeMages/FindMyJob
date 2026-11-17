//! Coordonnées de l'expéditeur que le profil peut ne pas renseigner (e-mail,
//! téléphone, liens). Une coordonnée vide dans le profil disparaît du
//! document, jamais remplacée par la valeur d'exemple de la référence ; si
//! la référence l'affichait, l'utilisateur en est averti. Commun au CV et à
//! la lettre : un seul avertissement par coordonnée.

// Importation des modules internes
use crate::docgen::identity::{display_url, text};
use crate::models::{Identity, UserView};

/// Coordonnée optionnelle de l'identité.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactField {
    Email,
    Telephone,
    Linkedin,
    Github,
    Portfolio,
}

impl ContactField {
    /// Toutes les coordonnées, dans l'ordre des avertissements.
    pub const ALL: [ContactField; 5] = [
        ContactField::Email,
        ContactField::Telephone,
        ContactField::Linkedin,
        ContactField::Github,
        ContactField::Portfolio,
    ];

    /// Nom français de la coordonnée.
    pub fn french_name(self) -> &'static str {
        match self {
            ContactField::Email => "adresse e-mail",
            ContactField::Telephone => "numéro de téléphone",
            ContactField::Linkedin => "lien LinkedIn",
            ContactField::Github => "lien GitHub",
            ContactField::Portfolio => "lien de portfolio",
        }
    }

    /// Indique si la référence affiche cette coordonnée (valeur d'exemple non vide).
    ///
    /// # Arguments
    ///
    /// * `example` - Identité des données d'exemple de la référence.
    ///
    /// # Returns
    ///
    /// Vrai si l'exemple a une valeur pour cette coordonnée.
    pub fn shown_in(self, example: &Identity) -> bool {
        let value = match self {
            ContactField::Email => &example.email,
            ContactField::Telephone => &example.telephone,
            ContactField::Linkedin => &example.linkedin,
            ContactField::Github => &example.github,
            ContactField::Portfolio => &example.portfolio,
        };
        !value.trim().is_empty()
    }

    /// Indique si le profil n'a aucune valeur pour cette coordonnée.
    ///
    /// # Arguments
    ///
    /// * `user` - Utilisateur du profil.
    ///
    /// # Returns
    ///
    /// Vrai si le profil est vide (ou blanc) sur cette coordonnée.
    pub fn is_empty_in(self, user: &UserView) -> bool {
        match self {
            ContactField::Email => text(user.email.as_ref()).is_empty(),
            ContactField::Telephone => text(user.telephone.as_ref()).is_empty(),
            ContactField::Linkedin => display_url(user.linkedin.as_ref()).is_empty(),
            ContactField::Github => display_url(user.github.as_ref()).is_empty(),
            ContactField::Portfolio => display_url(user.portfolio.as_ref()).is_empty(),
        }
    }

    /// Avertissement français affiché quand la coordonnée est retirée. Il ne
    /// nomme pas le document : il vaut pour le CV comme pour la lettre.
    pub fn warning(self) -> String {
        format!(
            "Le profil n'a pas de {} : la ligne correspondante a été retirée des documents. \
             Renseigne-le dans le profil pour qu'il y figure.",
            self.french_name()
        )
    }
}

/// Coordonnées que la référence affiche et que le profil ne renseigne pas,
/// sur l'ensemble des références (CV et lettre), sans doublon.
///
/// # Arguments
///
/// * `examples` - Identités d'exemple des références utilisées.
/// * `user` - Utilisateur du profil.
///
/// # Returns
///
/// Coordonnées retirées, dans l'ordre de `ContactField::ALL`.
pub fn missing_contacts(examples: &[&Identity], user: &UserView) -> Vec<ContactField> {
    ContactField::ALL
        .into_iter()
        .filter(|field| field.is_empty_in(user) && examples.iter().any(|example| field.shown_in(example)))
        .collect()
}
