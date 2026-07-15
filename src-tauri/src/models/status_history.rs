//! DTO représentant une entrée de l'historique de statut d'une candidature.

#[derive(serde::Serialize)]
pub struct StatusHistoryView {
    pub id: i64,
    pub candidature_id: i64,
    pub ancien_status: Option<String>,
    pub nouveau_status: String,
    pub changed_at: String,
}
