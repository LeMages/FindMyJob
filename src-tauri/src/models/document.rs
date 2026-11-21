//! DTO représentant un document de référence (CV ou lettre de motivation)
//! stocké sur le disque au format Typst (.typ) et compilé en PDF.

// Importation des crates externes
use chrono::{DateTime, Utc};

// Importation des modules internes
use crate::models::ReferenceIssue;

/// Référence courante vue par le frontend.
///
/// # Attributes
///
/// * `file_path` - Chemin relatif du `.typ` importé.
/// * `pdf_path` - Chemin relatif du PDF de l'exemple.
/// * `updated_at` - Date du dernier upload ou de la dernière restauration.
/// * `warnings` - Avertissements non bloquants de la validation, remplis
///   seulement au retour d'un import (vides à la lecture).
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DocumentReferenceView {
    pub file_path: String,
    pub pdf_path: String,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub warnings: Vec<ReferenceIssue>,
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
/// candidature liée à cette session (voir `services::candidatures::create`).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DocumentsStatus {
    pub cv_ready: bool,
    pub lm_ready: bool,
    pub all_ready: bool,
}

/// Fichiers de génération d'une référence validée par le contrat
/// FindMyJob, rangés dans le dossier de sa version.
///
/// # Attributes
///
/// * `template_path` - Chemin du template, relatif au dossier de données : le
///   `reference.typ` lui-même (une version antérieure au contrat pointe vers
///   un ancien `template.typ` et doit être réimportée).
/// * `example_data_path` - Chemin de `contract.json` (contrat extrait à
///   l'import : exemple, consignes, emplacements, format des périodes).
/// * `page_count` - Nombre de pages du rendu de l'exemple.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TemplateArtifacts {
    pub template_path: String,
    pub example_data_path: String,
    pub page_count: i32,
}

/// Référence courante (CV ou lettre de motivation) d'une session : la ligne
/// `documents` n'est qu'un pointeur vers une version de `document_versions`,
/// ce modèle en est la reconstruction par jointure (fichiers et contrat
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
/// * `template_path` - Chemin du template (voir `TemplateArtifacts`), `None` pour une version ancienne.
/// * `example_data_path` - Chemin de `contract.json`, `None` pour une version ancienne.
/// * `page_count` - Nombre de pages du rendu de l'exemple, `None` pour une version ancienne.
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
    /// Fichiers de génération de la référence, s'ils existent tous.
    ///
    /// # Returns
    ///
    /// `Some(TemplateArtifacts)` si la version les a tous, `None` sinon.
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
            warnings: Vec::new(),
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
