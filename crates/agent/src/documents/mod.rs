//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) et des polices utilisateur : compilation Typst, templatisation
//! des références et stockage sur disque.

pub mod fonts;
pub mod reference_versions;
pub mod storage;
pub mod templatize;
pub mod typst;
pub mod typst_diagnostics;
pub mod typst_text;

pub use fonts::{delete_font, import_font_folder, list_fonts, resolve_fonts_dir};
pub use reference_versions::{read_latest_version_pdf, read_version_pdf, restore_version};
// `SAVE_STEPS_TOTAL` implémente l'export local vers le dossier de téléchargements
// de l'utilisateur (flux desktop "postuler") : non exposé par l'API HTTP de cette
// étape (pas de notion de "dossier de téléchargements" sur un serveur headless),
// mais conservé pour un futur appelant desktop.
#[allow(unused_imports)]
pub use storage::{
    compile_reference, delete_generated_documents, document_info, get_reference, read_document_base64,
    read_document_bytes, save_generated_document, save_generation_trace, save_reference, SAVE_STEPS_TOTAL,
};
pub use templatize::retemplatize_current;
