//! Application des remplacements ciblés proposés par le LLM sur le
//! template précédent (correction d'un extrait plutôt que réécriture).

// Importation des modules internes
use crate::llm::templatize_template::TemplateEdit;

/// Applique des remplacements, dans l'ordre. Chaque texte à remplacer doit
/// apparaître exactement une fois dans le template courant.
///
/// # Arguments
///
/// * `template` - Template à corriger.
/// * `edits` - Remplacements.
///
/// # Returns
///
/// Le template corrigé, ou les remplacements inapplicables (aucun n'est
/// alors appliqué).
pub fn apply_edits(template: &str, edits: &[TemplateEdit]) -> Result<String, Vec<String>> {
    let mut current = template.to_string();
    let mut problems = Vec::new();
    for (index, edit) in edits.iter().enumerate() {
        let before = edit.before.trim_end();
        let occurrences = if before.is_empty() { 0 } else { current.matches(before).count() };
        if occurrences == 1 {
            current = current.replacen(before, edit.after.trim_end(), 1);
        } else {
            let first = before.lines().next().unwrap_or_default();
            problems.push(format!(
                "Le remplacement n°{} ne s'applique pas : son texte AVANT (« {first} … ») apparaît {occurrences} fois \
                 dans le template au lieu d'une. Recopie les lignes exactes, sans numéros, ou renvoie le template complet.",
                index + 1
            ));
        }
    }
    if problems.is_empty() {
        Ok(current)
    } else {
        Err(problems)
    }
}
