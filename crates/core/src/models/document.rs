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

/// Fichiers produits par la templatisation d'une référence (voir
/// `llm::templatize`), rangés à côté de son `reference.typ`.
///
/// # Attributes
///
/// * `template_path` - Chemin du `template.typ` (lit `data.json`), relatif au dossier de données.
/// * `example_data_path` - Chemin de `example_data.json`, relatif au dossier de données (contenu de
///   la référence au schéma canonique, avec la langue détectée).
/// * `page_count` - Nombre de pages du PDF de la référence.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TemplateArtifacts {
    pub template_path: String,
    pub example_data_path: String,
    pub page_count: i32,
}

/// Référence courante (CV ou lettre de motivation) d'une session : la ligne
/// `documents` n'est qu'un pointeur vers une version de `document_versions`,
/// ce modèle en est la reconstruction par jointure (fichiers et templatisation
/// lus sur la version pointée).
///
/// # Attributes
///
/// * `session_id` - Identifiant de la session propriétaire.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `current_version_id` - Version pointée dans `document_versions`.
/// * `original_filename` - Nom du fichier tel qu'uploadé par l'utilisateur.
/// * `file_path` - Chemin de `reference.typ`, relatif au dossier de données.
/// * `pdf_path` - Chemin de `reference.pdf`, relatif au dossier de données.
/// * `template_path` - Chemin du `template.typ`, `None` si la version n'est pas templatisée.
/// * `example_data_path` - Chemin de `example_data.json`, `None` si la version n'est pas templatisée.
/// * `page_count` - Nombre de pages du PDF de la référence, `None` si la version n'est pas templatisée.
/// * `updated_at` - Date du dernier changement de version courante (upload ou restauration).
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Document {
    pub session_id: i64,
    pub doc_type: String,
    pub current_version_id: i64,
    pub original_filename: String,
    pub file_path: String,
    pub pdf_path: String,
    pub template_path: Option<String>,
    pub example_data_path: Option<String>,
    pub page_count: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    /// Fichiers de templatisation de la référence, s'ils existent tous.
    ///
    /// # Returns
    ///
    /// `Some(TemplateArtifacts)` si la référence est templatisée, `None` sinon.
    pub fn template(&self) -> Option<TemplateArtifacts> {
        Some(TemplateArtifacts {
            template_path: self.template_path.clone()?,
            example_data_path: self.example_data_path.clone()?,
            page_count: self.page_count?,
        })
    }
}

/// Vue frontend de la référence courante : `updated_at` reflète le dernier
/// upload ou la dernière restauration de version.
impl From<Document> for DocumentReferenceView {
    fn from(doc: Document) -> Self {
        DocumentReferenceView {
            file_path: doc.file_path,
            pdf_path: doc.pdf_path,
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
/// * `typst_path` - Chemin du `.typ` de cette version, relatif au dossier de données.
/// * `pdf_path` - Chemin du `.pdf` de cette version, relatif au dossier de données.
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
