//! DTOs liés aux expériences professionnelles du profil.

use chrono::NaiveDate;

#[derive(serde::Serialize)]
pub struct ExperienceView {
    pub id: i64,
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
    pub type_contrat: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateExperienceInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
    pub type_contrat: Option<String>,
}
