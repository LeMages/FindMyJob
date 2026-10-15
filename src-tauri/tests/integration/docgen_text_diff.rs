//! Tests de la comparaison de textes rendus
//! (`findmyjob_lib::docgen::text_diff`).

// Importation des modules internes
use findmyjob_lib::docgen::text_diff::{alphanumeric_words, diff_words};

#[test]
fn textes_identiques_aux_espaces_pres() {
    let diff = diff_words("Bonjour  le\nmonde", "Bonjour le monde").expect("diff");
    assert_eq!(diff.distance(), 0);
}

#[test]
fn apostrophes_typographiques_equivalentes() {
    let diff = diff_words("l’outil d’ingestion « x »", "l'outil d'ingestion « x »").expect("diff");
    assert_eq!(diff.distance(), 0);
}

#[test]
fn mots_manquants_et_en_trop() {
    let diff = diff_words("a b c d", "a c d e").expect("diff");
    assert_eq!(diff.missing, vec!["b".to_string()]);
    assert_eq!(diff.extra, vec!["e".to_string()]);
}

#[test]
fn seuls_les_separateurs_ne_sont_pas_du_contenu() {
    assert!(alphanumeric_words("•  |  : – ,").is_empty());
    assert_eq!(alphanumeric_words("• Objet :"), vec!["Objet".to_string()]);
}

#[test]
fn separateurs_orphelins_detectes() {
    use findmyjob_lib::docgen::text_diff::orphan_separators;
    assert_eq!(orphan_separators("FindMyJob  |    |  2025"), vec!["| |".to_string()]);
    assert_eq!(orphan_separators("Anglais : | Français : natif"), vec![": |".to_string()]);
    assert!(orphan_separators("Need4Viz  |  Nov. 2025 – Avr. 2026 • Lyon").is_empty());
}

#[test]
fn ponctuation_de_bord_ignoree() {
    let diff = diff_words("Thyez, 74300 Thyez, le 9 octobre", "Thyez 74300 Thyez, le 9 octobre.").expect("diff");
    assert_eq!(diff.distance(), 0, "écart : {diff:?}");
    let diff = diff_words("« Objet : stage »", "Objet stage").expect("diff");
    assert_eq!(diff.missing, vec!["«".to_string(), ":".to_string(), "»".to_string()], "ponctuation seule comparée telle quelle");
}

#[test]
fn ecarts_cites_dans_leur_forme_d_origine() {
    let diff = diff_words("Thyez, 74300", "Lyon").expect("diff");
    assert_eq!(diff.missing, vec!["Thyez,".to_string(), "74300".to_string()]);
}
