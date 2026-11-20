//! Tests des consignes de rédaction de la référence : emplacements lus dans
//! le contrat (avec leur valeur d'exemple), linter de la prose et budgets.

// Importation des modules internes
use findmyjob_lib::docgen::budgets::{lm_budget, WordRange};
use findmyjob_lib::docgen::lint::{lint_prose, strip_dashes};
use findmyjob_lib::models::{LmData, ProseField, ProseGuidance, Slot, SlotMode};

/// Champ de prose.
fn field(path: &str, text: &str) -> ProseField {
    ProseField { path: path.to_string(), text: text.to_string() }
}

/// Emplacement déclaré.
fn slot(field: &str, mode: SlotMode) -> Slot {
    Slot { field: field.to_string(), mode, instruction: format!("Consigne de {field}") }
}

// ======================================================================
// Emplacements
// ======================================================================

#[test]
fn emplacements_avec_leur_valeur_d_exemple() {
    let example = LmData {
        subject: "Candidature".to_string(),
        paragraphs: vec!["Premier.".to_string(), "Second.".to_string()],
        ..Default::default()
    };
    let example = serde_json::to_value(&example).expect("JSON");
    let slots = [slot("subject", SlotMode::Variable), slot("paragraphs[1]", SlotMode::Adaptable)];

    let guidance = ProseGuidance::from_slots("  Ne jamais inventer. ", &slots, &example);

    assert_eq!(guidance.general, "Ne jamais inventer.");
    assert_eq!(guidance.zones.len(), 2);
    assert_eq!((guidance.zones[0].text.as_str(), guidance.zones[0].mode), ("Candidature", SlotMode::Variable));
    assert_eq!((guidance.zones[1].text.as_str(), guidance.zones[1].mode), ("Second.", SlotMode::Adaptable));
    assert_eq!(guidance.zones[1].instruction, "Consigne de paragraphs[1]");
    assert_eq!(guidance.zones_of("paragraphs[1]").count(), 1);
    assert_eq!(guidance.zones_of("paragraphs[0]").count(), 0, "sans emplacement : rédigé librement");
}

// ======================================================================
// Linter
// ======================================================================

#[test]
fn une_marque_de_la_reference_reste_interdite() {
    let fields = [field("subject", "Data Engineer – Stage")];
    let violations = lint_prose(&fields, "Objet : Stagiaire – Stage de fin d'études");
    assert_eq!(violations.len(), 1, "la référence n'exempte aucune marque : {violations:?}");
    assert_eq!(strip_dashes("a – b — c"), "a, b, c");
}

// ======================================================================
// Budgets
// ======================================================================

#[test]
fn budgets_reduits_proportionnellement() {
    let scaled = WordRange::around(100).scaled(0.5);
    assert_eq!(scaled.target, 50);

    let example = LmData { paragraphs: vec!["Un deux trois quatre.".to_string()], ..Default::default() };
    let budget = lm_budget(&example, 1);
    assert_eq!(budget.paragraph_count(), 1);
    assert!(budget.scaled(0.5).paragraph_words[0].target <= budget.paragraph_words[0].target);
}
