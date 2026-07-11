//! DTOs liés aux formations du profil.

#[derive(serde::Serialize)]
pub struct FormationView {
    pub id: i64,
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateFormationInput {
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}
