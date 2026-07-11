//! DTOs liés aux sessions de candidature.

#[derive(serde::Serialize)]
pub struct SessionView {
    pub id: i64,
    pub nom: String,
    pub candidature_count: i64,
    pub has_accepted: bool,
    pub created_at: String,
}

#[derive(serde::Deserialize)]
pub struct CreateSessionInput {
    pub nom: String,
}
