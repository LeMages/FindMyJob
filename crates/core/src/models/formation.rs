//! DTOs liés aux formations du profil.

use chrono::NaiveDate;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FormationView {
    pub id: i64,
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateFormationInput {
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
}
