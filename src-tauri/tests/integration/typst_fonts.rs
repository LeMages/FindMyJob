//! Tests : polices à la compilation Typst. Une police importée dans
//! `fonts_dir` est utilisée sans avertissement et embarquée dans le PDF ; une
//! famille introuvable fait échouer la compilation au lieu d'un repli
//! silencieux sur la police par défaut ; la police des références balisées
//! couvre tous leurs caractères, dans chaque langue.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::{copy_font_fixture, install_reference_font};
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::typst::{compile_contract, CompileFailure};

/// Noms `/BaseFont` des polices embarquées dans un PDF.
fn pdf_base_fonts(pdf: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(pdf);
    text.match_indices("/BaseFont/")
        .map(|(start, marker)| {
            text[start + marker.len()..].chars().take_while(|c| c.is_ascii_alphanumeric() || "+-_".contains(*c)).collect()
        })
        .collect()
}

#[test]
fn police_importee_utilisee_sans_avertissement() {
    let root = unique_temp_dir("typst-fonts-importee");
    let fonts = root.join("polices");
    copy_font_fixture("arimo", &fonts.join("arimo"));

    let doc = compile_contract("#set text(font: \"Arimo\")\nBonjour", None, &fonts)
        .expect("compilation avec la police importée");

    assert!(
        !doc.warnings.iter().any(|w| w.contains("unknown font family")),
        "avertissements : {:?}",
        doc.warnings
    );
    let base_fonts = pdf_base_fonts(&doc.pdf);
    assert!(base_fonts.iter().any(|f| f.ends_with("Arimo-Regular")), "polices du PDF : {base_fonts:?}");
    assert!(!base_fonts.iter().any(|f| f.contains("Libertinus")), "repli sur Libertinus : {base_fonts:?}");
}

/// Garde-fou du sous-ensemble `fonts/arimo_reference/` : chaque référence
/// balisée qui demande Arimo compile sans avertissement de police, dans
/// chaque langue, et chaque glyphe vient d'Arimo. Un caractère absent du
/// sous-ensemble ferait apparaître une police de repli dans le PDF au lieu
/// de passer en silence.
#[test]
fn references_balisees_entierement_rendues_en_arimo() {
    for reference in ["tagged/cv_balise.typ", "tagged/lm_balisee.typ"] {
        let root = unique_temp_dir("typst-fonts-arimo-reference");
        let fonts = root.join("polices");
        install_reference_font(&fonts);

        let source = read_reference(reference);
        assert!(source.contains("\"Arimo\""), "{reference} doit demander Arimo");
        let example = compile_contract(&source, None, &fonts)
            .unwrap_or_else(|e| panic!("{reference} : {}", e.message()))
            .metadata[0]
            .clone()
            .expect("métadonnée")["example"]
            .clone();
        for lang in ["fr", "en"] {
            let mut data = example.clone();
            data["lang"] = serde_json::Value::String(lang.to_string());
            let doc = compile_contract(&source, Some(&data.to_string()), &fonts)
                .unwrap_or_else(|e| panic!("{reference} ({lang}) : {}", e.message()));

            let font_warnings: Vec<_> = doc.warnings.iter().filter(|w| w.contains("font")).collect();
            assert!(font_warnings.is_empty(), "{reference} ({lang}) : avertissements de police {font_warnings:?}");
            let base_fonts = pdf_base_fonts(&doc.pdf);
            assert!(!base_fonts.is_empty(), "{reference} ({lang}) : aucune police dans le PDF");
            let fallbacks: Vec<_> = base_fonts.iter().filter(|f| !f.contains("Arimo-")).collect();
            assert!(fallbacks.is_empty(), "{reference} ({lang}) : glyphes de repli {fallbacks:?} ({base_fonts:?})");
        }
    }
}

#[test]
fn police_inexistante_erreur_bloquante_qui_nomme_la_famille() {
    let root = unique_temp_dir("typst-fonts-inexistante");

    let error = compile_contract("#set text(font: \"PoliceInexistante\")\nBonjour", None, &root.join("polices"))
        .err()
        .expect("erreur attendue");

    assert_eq!(error, CompileFailure::MissingFonts(vec!["PoliceInexistante".to_string()]));
    assert_eq!(error.message(), "Police introuvable : \"PoliceInexistante\". Importez-la dans Paramètres > Polices.");
}

#[test]
fn police_inexistante_avec_des_donnees_erreur_bloquante() {
    let root = unique_temp_dir("typst-fonts-template");
    let template = "#let data = if \"data\" in sys.inputs { json(bytes(sys.inputs.data)) } else { (name: \"\") }\n\
                    #set text(font: \"Inter\")\n#text(data.name)";

    let error = compile_contract(template, Some(r#"{"name": "x"}"#), &root.join("polices")).err().expect("erreur attendue");

    assert!(error.message().contains("Police introuvable : \"Inter\""), "message : {}", error.message());
}

#[test]
fn autre_avertissement_non_bloquant_mais_remonte() {
    let root = unique_temp_dir("typst-fonts-avertissement");

    let doc = compile_contract("#show par: set block(spacing: 1em)\nBonjour", None, &root.join("polices"))
        .expect("compilation non bloquée");

    assert!(doc.warnings.iter().any(|w| w.contains("no effect")), "avertissements : {:?}", doc.warnings);
}
