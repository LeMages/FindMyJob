pub mod fonts;
pub mod storage;

pub use fonts::{delete_font, import_font_folder, list_fonts, resolve_fonts_dir};
pub use storage::{
    compile_reference, document_info, export_for_apply, export_for_apply_with_message,
    finalize_candidature_document, get_reference, read_document_base64,
    save_generated_document, save_reference,
};
