//! Tests : bloc destinataire de la lettre, sur des rendus Typst réels. La
//! lettre balisée rend son destinataire à la largeur de son contenu (`box`),
//! calé contre la marge droite avec 2 comme avec 4 lignes, à la même hauteur
//! qu'un bloc de largeur fixe (`block(width: 7cm)`) ; le bloc de largeur fixe
//! est détecté quand le destinataire est réduit à l'entreprise et la ville.

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::shared::fake_llm::read_reference;
use crate::shared::font_fixtures::install_reference_font;
use crate::shared::temp_dir::unique_temp_dir;
use findmyjob_lib::documents::recipient_check::{recipient_misalignment, sparse_recipient_data};
use findmyjob_lib::documents::typst::{compile_contract, CompiledDocument};

/// Ouverture du bloc destinataire de la lettre balisée.
const FITTED_BLOCK: &str = "#align(right, box({";
/// Même bloc, de largeur fixe.
const FIXED_BLOCK: &str = "#align(right, block(width: 7cm, {";
/// Tolérance (pt) sur les positions comparées.
const TOLERANCE_PT: f64 = 0.5;

/// Source de la lettre balisée, bloc destinataire remplacé par `block`.
fn source(block: &str) -> String {
    let source = read_reference("tagged/lm_balisee.typ").replace("\r\n", "\n");
    assert!(source.contains(FITTED_BLOCK), "bloc destinataire introuvable");
    source.replace(FITTED_BLOCK, block)
}

/// Valeurs d'exemple de la lettre balisée, avec `extra` lignes supplémentaires.
fn example(extra: usize) -> Value {
    let doc = render(&source(FITTED_BLOCK), None);
    let mut data = doc.metadata[0].clone().expect("métadonnée")["example"].clone();
    let lines: Vec<Value> =
        data["recipient"]["extra_lines"].as_array().expect("liste").iter().take(extra).cloned().collect();
    data["recipient"]["extra_lines"] = Value::Array(lines);
    data
}

/// Compile une source avec des données (ou ses valeurs d'exemple).
fn render(source: &str, data: Option<&Value>) -> CompiledDocument {
    let fonts = unique_temp_dir("recipient-check").join("polices");
    install_reference_font(&fonts);
    let json = data.map(Value::to_string);
    compile_contract(source, json.as_deref(), &fonts).unwrap_or_else(|e| panic!("compilation : {}", e.message()))
}

/// Rendu avec le destinataire clairsemé (entreprise et ville seulement).
fn sparse_render(source: &str) -> CompiledDocument {
    let bytes = sparse_recipient_data(&serde_json::to_vec(&example(2)).expect("sérialisation")).expect("sparse");
    render(source, Some(&serde_json::from_slice(&bytes).expect("JSON")))
}

/// `(baseline, left, right)` des lignes du destinataire, de `first` à `last`.
fn recipient_lines(doc: &CompiledDocument, first: &str, last: &str) -> Vec<(f64, f64, f64)> {
    let position = |text: &str| doc.lines.iter().position(|l| l.words.join(" ") == text).expect("ligne du destinataire");
    doc.lines[position(first)..=position(last)].iter().map(|l| (l.baseline, l.left, l.right)).collect()
}

#[test]
fn box_cale_a_droite_a_la_meme_hauteur_qu_un_bloc_fixe_avec_2_et_4_lignes() {
    for extra in [2, 0] {
        let data = example(extra);
        let fixed = render(&source(FIXED_BLOCK), Some(&data));
        let fitted = render(&source(FITTED_BLOCK), Some(&data));
        let last = "1201 Genève";
        let before = recipient_lines(&fixed, "Exemple SA", last);
        let after = recipient_lines(&fitted, "Exemple SA", last);

        assert_eq!(before.len(), after.len());
        for (a, b) in before.iter().zip(&after) {
            assert!((a.0 - b.0).abs() <= TOLERANCE_PT, "ligne à {} pt contre {} pt ({extra} extra)", a.0, b.0);
        }
        // Le texte qui suit (ligne de date) est à la même hauteur : pas de décalage vertical.
        let date = |doc: &CompiledDocument| {
            doc.lines.iter().find(|l| l.words.contains(&"octobre".to_string())).expect("ligne de date").baseline
        };
        assert!((date(&fixed) - date(&fitted)).abs() <= TOLERANCE_PT, "{extra} extra");
        assert_eq!(fixed.page_count, fitted.page_count);
        // Calé contre la marge droite : la ligne la plus longue finit à la marge.
        let margin = fitted.lines.iter().map(|l| l.right).fold(f64::MIN, f64::max);
        let block_right = after.iter().map(|l| l.2).fold(f64::MIN, f64::max);
        assert!((margin - block_right).abs() <= TOLERANCE_PT, "bloc à {block_right} pt, marge à {margin} pt");
        // Texte aligné à gauche dans le bloc : même bord gauche pour toutes les lignes.
        assert!(after.iter().all(|l| (l.1 - after[0].1).abs() <= TOLERANCE_PT), "{after:?}");
    }
}

#[test]
fn largeur_fixe_detectee_box_acceptee() {
    let data = example(2);

    let fixed = source(FIXED_BLOCK);
    let misaligned = recipient_misalignment(&render(&fixed, Some(&data)), &data, &sparse_render(&fixed));
    let (block_right, margin) = misaligned.expect("bloc de largeur fixe décalé");
    assert!(margin - block_right > TOLERANCE_PT, "bloc à {block_right} pt, marge à {margin} pt");

    let fitted = source(FITTED_BLOCK);
    assert_eq!(recipient_misalignment(&render(&fitted, Some(&data)), &data, &sparse_render(&fitted)), None);
}

#[test]
fn bloc_de_la_reference_a_gauche_sans_objet() {
    // Le contrôle ne s'applique que si le bloc de la référence est à droite.
    let data = example(2);
    let left = source("#align(left, block(width: 7cm, {");
    assert_eq!(recipient_misalignment(&render(&left, Some(&data)), &data, &sparse_render(&left)), None);
}
