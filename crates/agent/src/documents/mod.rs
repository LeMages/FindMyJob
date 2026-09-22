//! Ré-exports publics de la couche de gestion des documents (CV, lettre de
//! motivation) et des polices utilisateur : compilation Typst et stockage sur disque.

pub mod fonts;
pub mod storage;
pub mod typst;

pub use fonts::{delete_font, import_font_folder, list_fonts, resolve_fonts_dir};
// `SAVE_STEPS_TOTAL` implémente l'export local vers le dossier de téléchargements
// de l'utilisateur (flux desktop "postuler") : non exposé par l'API HTTP de cette
// étape (pas de notion de "dossier de téléchargements" sur un serveur headless),
// mais conservé pour un futur appelant desktop.
#[allow(unused_imports)]
pub use storage::{
    compile_reference, document_info, get_reference, read_document_base64,
    save_generated_document, save_reference, SAVE_STEPS_TOTAL,
};
pub use typst::compile_to_pdf;
