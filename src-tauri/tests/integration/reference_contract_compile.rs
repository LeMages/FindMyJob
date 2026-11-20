//! Tests : compilation d'une référence au contrat. Les données passent par
//! `sys.inputs.data` (lues comme du texte, jamais évaluées), la métadonnée
//! `<findmyjob>` est lue par l'introspecteur, et une image collée dans le
//! fichier (`image(bytes(...))`) est acceptée là où une image par chemin est
//! refusée sans compilation.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::reference_contract::validate_reference;
use findmyjob_lib::documents::typst::compile_contract;
use findmyjob_lib::error::AppError;

/// Dossier de polices propre au test, avec la police de référence.
fn fonts(name: &str) -> std::path::PathBuf {
    let fonts = unique_temp_dir(name).join("polices");
    install_reference_font(&fonts);
    fonts
}

#[test]
fn metadonnee_lue_et_donnees_injectees() {
    let source = read_reference("lm_reference.typ");
    let fonts = fonts("contrat-compile");

    let base = compile_contract(&source, None, &fonts).expect("compilation avec l'exemple");
    assert_eq!(base.metadata.len(), 1);
    let metadata = base.metadata[0].clone().expect("métadonnée sérialisable");
    assert_eq!(metadata["contract"], "findmyjob/lettre");
    assert!(base.text.contains("Camille"), "{}", base.text);

    // Données injectées : une valeur en markup reste du texte.
    let mut data = metadata["example"].clone();
    data["identity"]["prenom"] = "*Alex* #panic()".into();
    data["lang"] = "en".into();
    let doc = compile_contract(&source, Some(&data.to_string()), &fonts).expect("compilation avec des données");
    assert!(doc.text.contains("*Alex*"), "{}", doc.text);
    assert!(doc.text.contains("Dear Sir or Madam,"), "variante anglaise : {}", doc.text);
}

#[tokio::test]
async fn image_collee_acceptee_image_par_chemin_refusee() {
    let source = read_reference("lm_reference.typ").replace("\r\n", "\n");
    let svg = "#image(bytes(\"<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10'><rect width='10' height='10'/></svg>\"), width: 4mm)\n";
    let anchor = "#block(L.salutation)";
    assert!(source.contains(anchor));

    let pasted = source.replacen(anchor, &format!("{svg}{anchor}"), 1);
    let validated = validate_reference("logo.typ", &pasted, "lettre_motivation", &fonts("contrat-svg")).await;
    assert!(validated.is_ok(), "{:?}", validated.err());

    let by_path = source.replacen(anchor, &format!("#image(\"logo.svg\")\n{anchor}"), 1);
    let refused = validate_reference("logo.typ", &by_path, "lettre_motivation", &fonts("contrat-chemin")).await;
    match refused {
        Err(AppError::InvalidReference(issues)) => {
            assert_eq!(issues.len(), 1, "{issues:?}");
            assert_eq!(issues[0].code.code(), "forbidden_construct");
        }
        other => panic!("refus attendu : {other:?}"),
    }
}
