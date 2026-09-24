//! DTO représentant un document de référence (CV ou lettre de motivation)
//! stocké sur le disque au format Typst (.typ) et compilé en PDF.

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DocumentReferenceView {
    pub file_path: String,
    pub pdf_path: String,
    pub updated_at: DateTime<Utc>,
}

/// État sur disque d'un document joint à une candidature (CV / lettre de
/// motivation), utilisé pour afficher honnêtement sa présence dans la page
/// de détail — le fichier peut avoir été déplacé ou supprimé après coup.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DocumentFileInfo {
    pub exists: bool,
    pub size: i64,
}

/// État de préparation des documents de référence d'une session (CV et
/// lettre de motivation) : `all_ready` conditionne la création d'une
/// candidature liée à cette session (voir `api::candidatures::create_candidature`).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DocumentsStatus {
    pub cv_ready: bool,
    pub lm_ready: bool,
    pub all_ready: bool,
}

/// Modèle DB d'un document de référence (CV ou lettre de motivation) tracké
/// dans la table `documents`. Le filesystem reste la source de vérité du
/// contenu ; ce modèle ne porte que les métadonnées et le cache d'analyse
/// de style LLM (`style_analysis`, rempli plus tard par le pipeline IA).
///
/// # Attributes
///
/// * `id` - Identifiant du document.
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `original_filename` - Nom du fichier tel qu'uploadé par l'utilisateur.
/// * `file_path` - Chemin absolu vers `reference.typ` sur le disque.
/// * `pdf_path` - Chemin absolu vers `reference.pdf`, si compilé.
/// * `style_analysis` - Cache JSON de l'analyse de style LLM (étape 0), `None` pour l'instant.
/// * `created_at` - Date de création (ISO 8601).
/// * `updated_at` - Date de dernière mise à jour (ISO 8601).
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Document {
    pub id: i64,
    pub session_id: i64,
    pub doc_type: String,
    pub original_filename: String,
    pub file_path: String,
    pub pdf_path: Option<String>,
    pub style_analysis: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
