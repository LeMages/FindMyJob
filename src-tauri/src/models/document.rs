//! DTO représentant un document de référence (CV ou lettre de motivation)
//! stocké sur le disque au format Typst (.typ) et compilé en PDF.

#[derive(serde::Serialize, Clone)]
pub struct DocumentReferenceView {
    pub file_path: String,
    pub pdf_path: String,
    pub updated_at: String,
}

/// État sur disque d'un document joint à une candidature (CV / lettre de
/// motivation), utilisé pour afficher honnêtement sa présence dans la page
/// de détail — le fichier peut avoir été déplacé ou supprimé après coup.
#[derive(serde::Serialize)]
pub struct DocumentFileInfo {
    pub exists: bool,
    pub size: i64,
}
