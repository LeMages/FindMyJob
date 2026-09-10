//! DTOs liés aux compétences du profil.

#[derive(serde::Deserialize)]
pub struct CreateSkillInput {
    pub nom: String,
    pub domaine: String,
}

#[derive(serde::Serialize)]
pub struct SkillView {
    pub id: i64,
    pub domaine: String,
    pub nom: String,
}
