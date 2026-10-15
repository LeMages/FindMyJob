//! DTOs liés aux langues du profil.

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateLangInput {
    pub nom: String,
    pub level: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct LangView {
    pub id: i64,
    pub nom: Option<String>,
    pub level: Option<String>,
}
