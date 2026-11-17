//! Tests : bloc destinataire de la lettre, sur des rendus Typst réels. La
//! référence balisée place son destinataire à droite (`block(width: 7cm)`) ;
//! le template conforme le rend à la largeur de son contenu (`box`), à la
//! même hauteur que la référence, calé contre la marge droite avec 2 comme
//! avec 4 lignes ; le template à largeur fixe est détecté.

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::recipient_check::{recipient_problems, sparse_recipient_data};
use findmyjob_lib::documents::typst::{compile_template, CompiledDocument};

/// Template à largeur fixe (comme la référence) et template conforme (`box`).
const FIXED_WIDTH: &str = "tagged/lm_balisee_template_largeur_fixe.typ";
const FITTED: &str = "tagged/lm_balisee_template.typ";
/// Tolérance (pt) sur les positions comparées.
const TOLERANCE_PT: f64 = 0.5;

/// Dossier de compilation avec la police de référence.
fn workspace() -> (PathBuf, PathBuf) {
    let root = unique_temp_dir("recipient-check");
    let fonts = root.join("polices");
    install_reference_font(&fonts);
    (root, fonts)
}

/// Données d'exemple de la lettre balisée, avec `extra` lignes supplémentaires.
fn example(extra: usize) -> Value {
    let mut data: Value = serde_json::from_str(&read_reference("tagged/lm_balisee_example_data.json")).expect("JSON");
    data["texts"] = serde_json::json!({ "t1": "P.J. : Curriculum vitae" });
    let lines: Vec<Value> =
        data["recipient"]["extra_lines"].as_array().expect("liste").iter().take(extra).cloned().collect();
    data["recipient"]["extra_lines"] = Value::Array(lines);
    data
}

/// Compile un template de fixture avec des données.
fn render(template: &str, data: &Value) -> Result<CompiledDocument, String> {
    let (root, fonts) = workspace();
    let source = read_reference(template).replace("\r\n", "\n");
    compile_template(source, serde_json::to_vec(data).expect("sérialisation"), root, fonts)
}

/// Rendu avec le destinataire clairsemé (entreprise et ville seulement).
fn sparse_render(template: &str) -> Result<CompiledDocument, String> {
    let bytes = sparse_recipient_data(&serde_json::to_vec(&example(4)).expect("sérialisation")).expect("sparse");
    render(template, &serde_json::from_slice(&bytes).expect("JSON"))
}

/// `(baseline, left, right)` des lignes du destinataire, de `first` à `last`.
fn recipient_lines(doc: &CompiledDocument, first: &str, last: &str) -> Vec<(f64, f64, f64)> {
    let position = |text: &str| doc.lines.iter().position(|l| l.words.join(" ") == text).expect("ligne du destinataire");
    doc.lines[position(first)..=position(last)].iter().map(|l| (l.baseline, l.left, l.right)).collect()
}

#[test]
fn box_cale_a_droite_a_la_meme_hauteur_que_la_reference_avec_2_et_4_lignes() {
    for extra in [2, 0] {
        let data = example(extra);
        let reference = render(FIXED_WIDTH, &data).expect("largeur fixe");
        let fitted = render(FITTED, &data).expect("box");
        let last = "1201 Genève";
        let before = recipient_lines(&reference, "Exemple SA", last);
        let after = recipient_lines(&fitted, "Exemple SA", last);

        assert_eq!(before.len(), after.len());
        for (a, b) in before.iter().zip(&after) {
            assert!((a.0 - b.0).abs() <= TOLERANCE_PT, "ligne à {} pt contre {} pt ({extra} extra)", a.0, b.0);
        }
        // Le texte qui suit (ligne de date) est à la même hauteur : pas de décalage vertical.
        let date = |doc: &CompiledDocument| {
            doc.lines.iter().find(|l| l.words.contains(&"octobre".to_string())).expect("ligne de date").baseline
        };
        assert!((date(&reference) - date(&fitted)).abs() <= TOLERANCE_PT, "{extra} extra");
        assert_eq!(reference.page_count, fitted.page_count);
        // Calé contre la marge droite : la ligne la plus longue finit à la marge.
        let margin = fitted.lines.iter().map(|l| l.right).fold(f64::MIN, f64::max);
        let block_right = after.iter().map(|l| l.2).fold(f64::MIN, f64::max);
        assert!((margin - block_right).abs() <= TOLERANCE_PT, "bloc à {block_right} pt, marge à {margin} pt");
        // Texte aligné à gauche dans le bloc : même bord gauche pour toutes les lignes.
        assert!(after.iter().all(|l| (l.1 - after[0].1).abs() <= TOLERANCE_PT), "{after:?}");
    }
}

#[test]
fn largeur_fixe_detectee_box_accepte() {
    let data = example(4);
    let with_example = render(FIXED_WIDTH, &data).expect("référence");

    let fixed = recipient_problems(&with_example, &data, &sparse_render(FIXED_WIDTH));
    assert_eq!(fixed.len(), 1, "{fixed:?}");
    assert!(fixed[0].contains("marge droite") && fixed[0].contains("box"), "{}", fixed[0]);

    let fitted_example = render(FITTED, &data).expect("box");
    assert!(recipient_problems(&fitted_example, &data, &sparse_render(FITTED)).is_empty());
}

#[test]
fn bloc_de_la_reference_a_gauche_sans_objet() {
    // Le contrôle ne s'applique que si le bloc de la référence est à droite.
    let data = example(4);
    let source = read_reference(FIXED_WIDTH)
        .replace("\r\n", "\n")
        .replace("#align(right, block(width: 7cm, {", "#align(left, block(width: 7cm, {");
    let (root, fonts) = workspace();
    let bytes = serde_json::to_vec(&data).expect("JSON");
    let with_example =
        compile_template(source.clone(), bytes.clone(), root.clone(), fonts.clone()).expect("rendu à gauche");
    let sparse = compile_template(source, sparse_recipient_data(&bytes).expect("sparse"), root, fonts);
    assert!(recipient_problems(&with_example, &data, &sparse).is_empty());
}
