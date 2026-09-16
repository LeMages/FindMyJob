//! DTOs liés à l'utilisateur unique de l'application (profil).

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateUserInput {
    pub nom: String,
    pub prenom: String,
    pub email: String,
    pub telephone: Option<String>,
    pub ville: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserView {
    pub id: i64,
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub ville: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateUserInfoInput {
    pub prenom: Option<String>,
    pub nom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub permis: Option<bool>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
}
