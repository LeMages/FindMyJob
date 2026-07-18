//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) : compilation Typst et stockage sur disque.

pub mod storage;
pub mod typst;

pub use storage::{
    compile_reference, document_info, finalize_candidature_document, get_reference,
    read_document_base64, resolve_reference_dir, save_generated_document, save_reference,
    select_candidature_document, upload_candidature_document,
};
pub use typst::compile_to_pdf;
