//! Tests des mots d'un texte rendu
//! (`findmyjob_lib::docgen::rendered_words`).

// Importation des modules internes
use findmyjob_lib::docgen::rendered_words::{normalize_quotes, orphan_separators, word_key, words};

#[test]
fn mots_separes_par_tout_espace() {
    assert_eq!(words("Bonjour  le
monde"), ["Bonjour", "le", "monde"]);
}

#[test]
fn apostrophes_typographiques_equivalentes() {
    assert_eq!(normalize_quotes("l’outil d’ingestion"), "l'outil d'ingestion");
}

#[test]
fn ponctuation_de_bord_ignoree() {
    assert_eq!(word_key("Lyon,"), "Lyon");
    assert_eq!(word_key("«"), "«", "ponctuation seule gardée telle quelle");
}

#[test]
fn separateurs_orphelins_detectes() {
    assert_eq!(orphan_separators("FindMyJob  |    |  2025"), vec!["| |".to_string()]);
    assert_eq!(orphan_separators("Anglais : | Français : natif"), vec![": |".to_string()]);
    assert!(orphan_separators("Transvia Logistique  |  Sept. 2025 – Févr. 2026 • Lyon").is_empty());
}
