//! DTO représentant la progression du pipeline de génération de documents
//! (CV + lettre de motivation) d'une candidature — consultée par polling
//! depuis l'interface pendant que l'application génère les documents en
//! tâche de fond (voir `crate::llm` côté core, `pipeline::generate_documents`
//! et `progress::ProgressStore` côté desktop).

/// Statut d'une génération de documents en cours de suivi.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStatus {
    InProgress,
    Completed,
    Failed,
}

/// Progression courante de la génération des documents d'une candidature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationProgress {
    pub candidature_id: i64,
    pub current_step: u8,
    pub total_steps: u8,
    pub step_label: String,
    pub status: GenerationStatus,
    pub error: Option<String>,
}
