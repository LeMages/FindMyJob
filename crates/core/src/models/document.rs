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
/// * `current_version_id` - Version courante dans `document_versions`, `None` si antérieure au versioning.
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
    pub current_version_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vue frontend de la référence courante, construite depuis sa ligne
/// `documents` : `updated_at` reflète le dernier upload ou la dernière
/// restauration de version.
impl From<Document> for DocumentReferenceView {
    fn from(doc: Document) -> Self {
        DocumentReferenceView {
            file_path: doc.file_path,
            pdf_path: doc.pdf_path.unwrap_or_default(),
            updated_at: doc.updated_at,
        }
    }
}

/// Version d'un document de référence (CV ou lettre de motivation) d'une
/// session, tracée dans `document_versions`. Chaque upload crée une version ;
/// une ancienne version peut être restaurée comme référence courante.
///
/// # Attributes
///
/// * `id` - Identifiant de la version (nomme aussi son dossier sur disque).
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `version` - Numéro d'affichage (1, 2, 3…) propre au couple session/doc_type.
/// * `original_filename` - Nom du fichier tel qu'uploadé par l'utilisateur.
/// * `typst_path` - Chemin absolu du `.typ` de cette version.
/// * `pdf_path` - Chemin absolu du `.pdf` de cette version.
/// * `created_at` - Date d'upload.
/// * `is_current` - Vrai si c'est la version utilisée par la génération.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DocumentVersionView {
    pub id: i64,
    pub session_id: i64,
    pub doc_type: String,
    pub version: i32,
    pub original_filename: String,
    pub typst_path: String,
    pub pdf_path: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}
