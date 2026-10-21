//! Tests des consignes de rédaction de la référence : cohérence des zones
//! avec la prose d'exemple, respect des zones `[FIXE]` à la rédaction,
//! linter et budgets qui épargnent le texte recopié.

// Importation des modules internes
use findmyjob_lib::docgen::budgets::{lm_budget, WordRange};
use findmyjob_lib::docgen::fixed_zones::{fixed_zone_violations, transform_outside_fixed, without_fixed_text};
use findmyjob_lib::docgen::lint::{lint_prose, strip_dashes};
use findmyjob_lib::docgen::words::count_words;
use findmyjob_lib::models::{LmData, ProseField, ProseGuidance, ProseZone, ZoneMode};

/// Champ de prose.
fn field(path: &str, text: &str) -> ProseField {
    ProseField { path: path.to_string(), text: text.to_string() }
}

/// Zone d'un champ.
fn zone(field: &str, mode: ZoneMode, text: &str) -> ProseZone {
    ProseZone { field: field.to_string(), mode, instruction: String::new(), text: text.to_string() }
}

/// Dernier paragraphe de la lettre de référence : une zone fixe.
const FIXED: &str = "Résidant à Thyez, à une quarantaine de minutes de Genève, je peux travailler en tant que frontalier.";

/// Consignes : paragraphe 0 découpé en une zone fixe puis une variable.
fn guidance() -> ProseGuidance {
    ProseGuidance {
        general: "Ne jamais inventer.".to_string(),
        zones: vec![
            zone("paragraphs[0]", ZoneMode::Fixe, FIXED),
            zone("paragraphs[0]", ZoneMode::Variable, "Je postule chez Exemple SA."),
        ],
    }
}

// ======================================================================
// Zones fixes (rédaction)
// ======================================================================

#[test]
fn zone_fixe_recopiee_avec_une_information_mise_a_jour() {
    let information = vec!["Thyez".to_string()];
    let written = FIXED.replace("Thyez", "Annecy") + " Je postule chez Lumen.";
    let violations = fixed_zone_violations(&guidance(), &[field("paragraphs[0]", &written)], &information);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn zone_fixe_reecrite_signalee() {
    let information = vec!["Thyez".to_string()];
    let written = "Je vis près de Genève et je peux être frontalier. Je postule chez Lumen.";
    let violations = fixed_zone_violations(&guidance(), &[field("paragraphs[0]", written)], &information);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("[FIXE]") && violations[0].message.contains("\"quarantaine\""));
}

#[test]
fn le_linter_ignore_le_texte_recopie() {
    let guidance = ProseGuidance { general: String::new(), zones: vec![zone("subject", ZoneMode::Fixe, "Stage – 6 mois")] };
    let fields = [field("subject", "Data Engineer, Stage – 6 mois")];
    assert!(lint_prose(&without_fixed_text(&guidance, &fields), "").is_empty());
    assert_eq!(lint_prose(&fields, "").len(), 1, "sans zone fixe, le tiret est relevé");
}

#[test]
fn une_marque_de_la_reference_reste_interdite() {
    let fields = [field("subject", "Data Engineer – Stage")];
    let violations = lint_prose(&fields, "Objet : Stagiaire – Stage de fin d'études");
    assert_eq!(violations.len(), 1, "la référence n'exempte aucune marque : {violations:?}");
    assert_eq!(strip_dashes("a – b — c"), "a, b, c");
}

#[test]
fn nettoyage_hors_zone_fixe() {
    let cleaned = transform_outside_fixed("Intro — suite. Stage – 6 mois. Fin — ok", &["Stage – 6 mois."], strip_dashes);
    assert_eq!(cleaned, "Intro, suite. Stage – 6 mois. Fin, ok");
}

// ======================================================================
// Budgets
// ======================================================================

#[test]
fn les_mots_fixes_ne_sont_jamais_reduits() {
    let range = WordRange::around(100).with_fixed(60);
    let scaled = range.scaled(0.5);
    assert_eq!(scaled.target, 80, "60 mots fixes + la moitié des 40 autres");
    assert_eq!(scaled.fixed, 60);

    let example = LmData { paragraphs: vec![format!("{FIXED} Je postule chez Exemple SA.")], ..Default::default() };
    let budget = lm_budget(&example, 1).with_fixed_words(&guidance());
    assert_eq!(budget.paragraph_words[0].fixed, count_words(FIXED));
    assert!(budget.scaled(0.5).paragraph_words[0].target >= count_words(FIXED));
}
