//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) et des polices utilisateur : compilation Typst, templatisation
//! des références et stockage sur disque.

pub mod doc_type;
pub mod fonts;
pub mod paragraph_check;
pub mod reference_versions;
pub mod sparse_check;
pub mod storage;
pub mod templatize;
pub mod templatize_debug;
pub mod typst;
pub mod typst_diagnostics;
pub mod typst_lines;
pub mod typst_text;

pub use doc_type::parse_doc_type;
pub use fonts::{delete_font, import_font_folder, list_fonts};
pub use reference_versions::{read_version_pdf, restore_version};
pub use storage::{
    compile_reference, delete_generated_documents, delete_session_documents, document_info, get_reference,
    read_document_base64, read_document_bytes, save_generated_document, save_generation_trace, save_reference,
};
