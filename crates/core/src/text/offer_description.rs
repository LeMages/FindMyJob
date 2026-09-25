//! Assemblage de la description finale d'une offre à partir de ses champs
//! sources, en sections titrées (« Descriptif du poste », « Profil
//! recherché », « Déroulement des entretiens »).

// Importation des modules internes
use super::html_to_plain::html_to_plain;

const TITLE_DESCRIPTION: &str = "Descriptif du poste";
const TITLE_PROFILE: &str = "Profil recherché";
const TITLE_RECRUITMENT_PROCESS: &str = "Déroulement des entretiens";

/// Construit la description texte d'une offre, section par section, en
/// omettant les sections vides. La présentation de l'entreprise n'est
/// volontairement pas incluse (hors du périmètre du poste).
///
/// # Arguments
///
/// * `description` - Descriptif du poste (HTML ou Markdown), si connu.
/// * `profile` - Profil recherché (HTML ou Markdown), si connu.
/// * `recruitment_process` - Déroulement des entretiens (HTML ou Markdown), si connu.
///
/// # Returns
///
/// Les sections non vides séparées par une ligne vide, chacune précédée de
/// son titre en texte simple ; `None` si toutes les sections sont vides.
pub fn assemble_offer_description(
    description: Option<&str>,
    profile: Option<&str>,
    recruitment_process: Option<&str>,
) -> Option<String> {
    let sections: Vec<String> = [
        (TITLE_DESCRIPTION, description),
        (TITLE_PROFILE, profile),
        (TITLE_RECRUITMENT_PROCESS, recruitment_process),
    ]
    .into_iter()
    .filter_map(|(title, raw)| raw.and_then(html_to_plain).map(|text| format!("{title}\n{text}")))
    .collect();

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}
