//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) et des polices utilisateur : compilation Typst et stockage sur disque.

pub mod fonts;
pub mod storage;
pub mod typst;

pub use fonts::{delete_font, import_font_folder, list_fonts, resolve_fonts_dir};
pub use storage::{
    build_export_dir_name, compile_reference, copy_candidature_documents, document_info,
    finalize_candidature_document, get_reference, read_document_base64,
    save_generated_document, save_reference, write_export_message, SAVE_STEPS_TOTAL,
};
pub use typst::compile_to_pdf;
