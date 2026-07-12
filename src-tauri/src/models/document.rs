//! DTO représentant un document de référence (CV ou lettre de motivation)
//! stocké sur le disque au format Typst (.typ) et compilé en PDF.

#[derive(serde::Serialize, Clone)]
pub struct DocumentReferenceView {
    pub file_path: String,
    pub pdf_path: String,
    pub updated_at: String,
}
