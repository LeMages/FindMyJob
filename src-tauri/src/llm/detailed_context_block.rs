//! Contexte détaillé des entrées retenues, pour la rédaction de la lettre
//! uniquement. Le bloc est distinct de `profile_facts_block`, partagé avec
//! le CV : le CV ne peut donc pas recevoir le contexte par erreur.
//!
//! Le contexte n'est jamais traduit : il est donné tel que saisi, avec la
//! consigne de rédiger la lettre dans la langue du document. Les consignes
//! sont écrites en phrases simples, sans tiret cadratin (`writing_rules`).

// Importation des modules internes
use crate::docgen::selection::Selection;
use crate::models::doc_lang::language_name;

/// Contexte détaillé d'une entrée retenue.
///
/// # Attributes
///
/// * `label` - Entrée concernée, repérée comme dans `profile_facts_block`.
/// * `text` - Contexte saisi, trimé.
#[derive(Clone, Debug, PartialEq)]
pub struct DetailedContext {
    pub label: String,
    pub text: String,
}

/// Contexte trimé, `None` s'il est absent ou blanc.
fn non_empty(contexte: Option<&str>) -> Option<String> {
    contexte.map(str::trim).filter(|c| !c.is_empty()).map(str::to_string)
}

/// Contextes détaillés transmis à la lettre : ceux de toutes les expériences
/// puis des projets retenus, dans l'ordre d'affichage, sans les vides.
///
/// # Arguments
///
/// * `selection` - Entrées retenues (toutes les expériences, les projets choisis).
///
/// # Returns
///
/// Contextes non vides ; liste vide si aucun n'est renseigné.
pub fn letter_contexts(selection: &Selection) -> Vec<DetailedContext> {
    let experiences = selection.experiences.iter().filter_map(|e| {
        non_empty(e.contexte.as_deref()).map(|text| DetailedContext { label: format!("Expérience id {} : {}", e.id, e.poste), text })
    });
    let projects = selection.projects.iter().filter_map(|p| {
        non_empty(p.contexte.as_deref()).map(|text| DetailedContext { label: format!("Projet id {} : {}", p.id, p.titre), text })
    });
    experiences.chain(projects).collect()
}

/// Bloc du contexte détaillé, à placer avant le modèle de style.
///
/// # Arguments
///
/// * `contexts` - Contextes retenus (`letter_contexts`).
/// * `target_lang` - Langue de la lettre.
///
/// # Returns
///
/// Bloc à insérer dans le message, `None` si aucun contexte n'est renseigné :
/// le prompt reste alors celui d'avant l'ajout du contexte.
pub fn detailed_context_block(contexts: &[DetailedContext], target_lang: &str) -> Option<String> {
    if contexts.is_empty() {
        return None;
    }
    let language = language_name(target_lang);
    let mut lines = vec![
        "CONTEXTE DÉTAILLÉ DU CANDIDAT".to_string(),
        "Ces notes du candidat complètent ses expériences et ses projets. C'est une matière de fond, pas un texte à citer.".to_string(),
        "N'en reprends aucune phrase : reformule chaque fait avec tes propres mots.".to_string(),
        "Ne retiens que les faits utiles à l'offre.".to_string(),
        "Le style reste celui du modèle de style.".to_string(),
        format!("Ces notes peuvent être écrites dans une autre langue que la lettre. La lettre reste entièrement rédigée en {language}."),
        "Tout ce qui figure dans ces notes, chiffres compris, est un fait du profil : tu peux l'utiliser dans la lettre.".to_string(),
    ];
    for context in contexts {
        lines.push(format!("\n{}\n<<<CONTEXTE\n{}\nCONTEXTE>>>", context.label, context.text));
    }
    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docgen::selection::select;
    use crate::llm::test_profile::{experience, profile_with, projet};

    #[test]
    fn aucun_contexte_aucun_bloc() {
        let profile = profile_with(vec![experience(1, None), experience(2, Some("  "))], vec![projet(3, None)]);
        let contexts = letter_contexts(&select(&profile, &[3]));
        assert!(contexts.is_empty());
        assert_eq!(detailed_context_block(&contexts, "fr"), None);
    }

    #[test]
    fn experiences_et_projets_retenus_seulement() {
        let profile = profile_with(
            vec![experience(1, Some(" CTX-EXP "))],
            vec![projet(2, Some("CTX-RETENU")), projet(3, Some("CTX-ECARTE"))],
        );
        let contexts = letter_contexts(&select(&profile, &[2]));
        let texts: Vec<&str> = contexts.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, ["CTX-EXP", "CTX-RETENU"]);
        assert_eq!(contexts[0].label, "Expérience id 1 : Poste 1");
        assert_eq!(contexts[1].label, "Projet id 2 : Projet 2");
    }

    #[test]
    fn consignes_sans_tiret_et_langue_du_document() {
        let contexts = vec![DetailedContext { label: "Projet id 2 : Projet 2".to_string(), text: "CTX".to_string() }];
        let block = detailed_context_block(&contexts, "en").expect("bloc");
        assert!(block.contains("rédigée en anglais"), "{block}");
        assert!(block.contains("<<<CONTEXTE\nCTX\nCONTEXTE>>>"), "{block}");
        for dash in ['\u{2014}', '\u{2013}'] {
            assert!(!block.contains(dash), "aucun tiret cadratin ni demi-cadratin : {block}");
        }
    }
}
