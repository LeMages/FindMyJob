//! Tests : les exemples livrés (`docs/reference-contract/cv_exemple.typ` et
//! `lm_exemple.typ`), copiés dans le prompt proposé à l'utilisateur, passent
//! le validateur du contrat sans aucun avertissement.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::reference_contract::validate_reference;

/// Lit un exemple de `docs/reference-contract/`.
fn read_example(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("docs").join("reference-contract").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("exemple {} illisible : {e}", path.display()))
}

#[tokio::test]
async fn exemples_livres_conformes_sans_avertissement() {
    for (file, doc_type) in [("cv_exemple.typ", "cv"), ("lm_exemple.typ", "lettre_motivation")] {
        let fonts = unique_temp_dir("exemples-livres").join("polices");
        install_reference_font(&fonts);

        let validated = validate_reference(file, &read_example(file), doc_type, &fonts)
            .await
            .unwrap_or_else(|e| panic!("{file} refusé : {e}"));

        assert!(validated.warnings.is_empty(), "{file} : {:?}", validated.warnings);
        assert_eq!(validated.contract.page_count, 1, "{file} tient sur une page");
    }
}
