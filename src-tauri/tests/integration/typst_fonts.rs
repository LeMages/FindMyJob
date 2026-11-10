//! Tests : polices à la compilation Typst. Une police importée dans
//! `fonts_dir` est utilisée sans avertissement et embarquée dans le PDF ; une
//! famille introuvable fait échouer la compilation au lieu d'un repli
//! silencieux sur la police par défaut ; la police des références balisées
//! couvre tous leurs caractères.

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::{copy_font_fixture, install_reference_font};
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::typst::{compile_document, compile_template};

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

    let source = "#set text(font: \"Arimo\")\nBonjour".to_string();
    let doc = compile_document(source, root.clone(), fonts).expect("compilation avec la police importée");

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
/// balisée qui demande Arimo compile sans avertissement de police, et chaque
/// glyphe vient d'Arimo. Un caractère absent du sous-ensemble ferait
/// apparaître une police de repli dans le PDF au lieu de passer en silence.
#[test]
fn references_balisees_entierement_rendues_en_arimo() {
    for reference in ["tagged/cv_balise.typ", "tagged/lm_balisee.typ"] {
        let root = unique_temp_dir("typst-fonts-arimo-reference");
        let fonts = root.join("polices");
        install_reference_font(&fonts);

        let source = read_reference(reference);
        assert!(source.contains("\"Arimo\""), "{reference} doit demander Arimo");
        let doc = compile_document(source, root.clone(), fonts).unwrap_or_else(|e| panic!("{reference} : {e}"));

        let font_warnings: Vec<_> = doc.warnings.iter().filter(|w| w.contains("font")).collect();
        assert!(font_warnings.is_empty(), "{reference} : avertissements de police {font_warnings:?}");
        let base_fonts = pdf_base_fonts(&doc.pdf);
        assert!(!base_fonts.is_empty(), "{reference} : aucune police dans le PDF");
        let fallbacks: Vec<_> = base_fonts.iter().filter(|f| !f.contains("Arimo-")).collect();
        assert!(fallbacks.is_empty(), "{reference} : glyphes de repli {fallbacks:?} (polices : {base_fonts:?})");
    }
}

#[test]
fn police_inexistante_erreur_bloquante_qui_nomme_la_famille() {
    let root = unique_temp_dir("typst-fonts-inexistante");
    let source = "#set text(font: \"PoliceInexistante\")\nBonjour".to_string();

    let error = compile_document(source, root.clone(), root.join("polices")).err().expect("erreur attendue");

    assert_eq!(error, "Police introuvable : \"PoliceInexistante\". Importez-la dans Paramètres > Polices.");
}

#[test]
fn police_inexistante_dans_un_template_erreur_bloquante() {
    let root = unique_temp_dir("typst-fonts-template");
    let template = "#let data = json(\"data.json\")\n#set text(font: \"Inter\")\n#text(data.name)".to_string();

    let error = compile_template(template, br#"{"name": "x"}"#.to_vec(), root.clone(), root.join("polices"))
        .err()
        .expect("erreur attendue");

    assert!(error.contains("Police introuvable : \"Inter\""), "message : {error}");
}

#[test]
fn autre_avertissement_non_bloquant_mais_remonte() {
    let root = unique_temp_dir("typst-fonts-avertissement");
    let source = "#show par: set block(spacing: 1em)\nBonjour".to_string();

    let doc = compile_document(source, root.clone(), root.join("polices")).expect("compilation non bloquée");

    assert!(doc.warnings.iter().any(|w| w.contains("no effect")), "avertissements : {:?}", doc.warnings);
}
