//! DTOs liés aux expériences professionnelles du profil.

#[derive(serde::Serialize)]
pub struct ExperienceView {
    pub id: i64,
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
    pub type_contrat: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateExperienceInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
    pub type_contrat: Option<String>,
}
