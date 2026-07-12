//! DTO représentant un contact recruteur rattaché à une candidature.

#[derive(serde::Serialize)]
pub struct ContactView {
    pub id: i64,
    pub candidature_id: i64,
    pub nom: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}
