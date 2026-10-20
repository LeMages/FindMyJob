//! DTOs liés à l'utilisateur unique de l'application (profil).

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateUserInput {
    pub nom: String,
    pub prenom: String,
    pub email: String,
    pub telephone: Option<String>,
    /// Ville du candidat.
    pub ville: Option<String>,
    /// Code postal du candidat.
    pub zip_code: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UserView {
    pub id: i64,
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    /// Ville du candidat.
    pub ville: Option<String>,
    /// Code postal du candidat.
    pub zip_code: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
    /// Centres d'intérêt, une ligne = un item.
    pub interets: Option<String>,
    /// Langue dans laquelle le profil est rédigé (`"fr"` ou `"en"`).
    pub langue_profil: String,
    pub created_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateUserInfoInput {
    pub prenom: Option<String>,
    pub nom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    #[serde(default)]
    pub ville: Option<String>,
    #[serde(default)]
    pub zip_code: Option<String>,
    pub permis: Option<bool>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
    /// Centres d'intérêt, une ligne = un item.
    #[serde(default)]
    pub interets: Option<String>,
}
