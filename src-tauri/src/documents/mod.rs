//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) : compilation Typst et stockage sur disque.

pub mod storage;
pub mod typst;

pub use storage::{
    compile_reference, document_info, export_for_apply, export_for_apply_with_message,
    finalize_candidature_document, get_reference, read_document_base64,
    save_generated_document, save_reference,
};
pub use typst::compile_to_pdf;
