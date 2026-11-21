//! Contrôle des emplacements rédigés déclarés par une référence (`slots`) :
//! champ de prose connu et présent dans l'exemple, mode valide, un seul
//! emplacement par champ.
//!
//! Champs de prose : `identity.title` et `summary` pour un CV ; `subject`
//! et `paragraphs[i]` pour une lettre (`i` < nombre de paragraphes de
//! l'exemple). Une prose sans emplacement déclaré est rédigée en mode
//! « variable ».

// Importation des crates externes
use serde_json::Value;

// Importation des modules internes
use crate::models::{RawSlot, ReferenceIssue, ReferenceIssueCode, Slot, SlotMode};

/// Champs de prose d'un CV.
const CV_PROSE_FIELDS: [&str; 2] = ["identity.title", "summary"];
/// Préfixe des paragraphes d'une lettre (`paragraphs[0]`…).
const PARAGRAPH_PREFIX: &str = "paragraphs[";

/// Vrai si `field` est un champ de prose du type de document, présent dans
/// l'exemple.
fn is_prose_field(field: &str, doc_type: &str, example: &Value) -> bool {
    if doc_type == "cv" {
        return CV_PROSE_FIELDS.contains(&field);
    }
    if field == "subject" {
        return true;
    }
    let Some(index) = field.strip_prefix(PARAGRAPH_PREFIX).and_then(|rest| rest.strip_suffix(']')) else {
        return false;
    };
    let count = example["paragraphs"].as_array().map_or(0, Vec::len);
    index.parse::<usize>().is_ok_and(|i| i < count)
}

/// Valide les emplacements déclarés.
///
/// # Arguments
///
/// * `raw` - Emplacements tels qu'écrits dans la métadonnée.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `example` - Valeurs d'exemple (déjà conformes au schéma).
///
/// # Returns
///
/// Emplacements valides, et un problème par emplacement refusé.
pub fn check_slots(raw: &[RawSlot], doc_type: &str, example: &Value) -> (Vec<Slot>, Vec<ReferenceIssue>) {
    let mut slots: Vec<Slot> = Vec::new();
    let mut issues = Vec::new();
    for (i, slot) in raw.iter().enumerate() {
        let field = slot.field.trim();
        if !is_prose_field(field, doc_type, example) {
            issues.push(
                ReferenceIssue::new(ReferenceIssueCode::SlotUnknownField)
                    .at_path(format!("slots[{i}].field"))
                    .with_detail(field),
            );
            continue;
        }
        let Some(mode) = SlotMode::parse(slot.mode.trim()) else {
            issues.push(
                ReferenceIssue::new(ReferenceIssueCode::SlotInvalidMode)
                    .at_path(format!("slots[{i}].mode"))
                    .with_detail(&slot.mode),
            );
            continue;
        };
        if slots.iter().any(|s| s.field == field) {
            issues.push(
                ReferenceIssue::new(ReferenceIssueCode::SlotDuplicated)
                    .at_path(format!("slots[{i}].field"))
                    .with_detail(field),
            );
            continue;
        }
        slots.push(Slot { field: field.to_string(), mode, instruction: slot.instruction.trim().to_string() });
    }
    (slots, issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn raw(field: &str, mode: &str) -> RawSlot {
        RawSlot { field: field.to_string(), mode: mode.to_string(), instruction: String::new() }
    }

    #[test]
    fn emplacements_de_la_lettre() {
        let example = json!({ "paragraphs": ["a", "b"] });
        let (slots, issues) = check_slots(
            &[
                raw("subject", "variable"),
                raw("paragraphs[1]", "adaptable"),
                raw("paragraphs[2]", "variable"),
                raw("summary", "variable"),
                raw("paragraphs[0]", "fixe"),
                raw("subject", "adaptable"),
            ],
            "lettre_motivation",
            &example,
        );
        assert_eq!(slots.len(), 2);
        let found: Vec<(ReferenceIssueCode, Option<String>)> =
            issues.into_iter().map(|i| (i.code, i.path)).collect();
        assert_eq!(
            found,
            vec![
                (ReferenceIssueCode::SlotUnknownField, Some("slots[2].field".to_string())),
                (ReferenceIssueCode::SlotUnknownField, Some("slots[3].field".to_string())),
                (ReferenceIssueCode::SlotInvalidMode, Some("slots[4].mode".to_string())),
                (ReferenceIssueCode::SlotDuplicated, Some("slots[5].field".to_string())),
            ]
        );
    }

    #[test]
    fn emplacements_du_cv() {
        let (slots, issues) =
            check_slots(&[raw("identity.title", "variable"), raw("summary", "adaptable")], "cv", &json!({}));
        assert_eq!(slots.len(), 2);
        assert!(issues.is_empty());
    }
}
