//! Nettoyage final des champs extraits d'une offre, commun aux trois chemins
//! (JSON-LD, LLM sur la page, texte collé) : quelle que soit la source, la
//! description renvoyée est du texte brut sans balise, sans emoji ni
//! décoration Markdown, avec ses titres et listes conservés.

// Importation des modules internes
use crate::llm::scraping_extract::ScrapedOfferFields;
use crate::text::html_to_plain;

/// Types de contrat acceptés par le formulaire de candidature.
const ALLOWED_CONTRACT_TYPES: &[&str] = &["stage", "alternance", "interim", "cdd", "cdi"];

/// Nettoie tous les champs extraits d'une offre.
///
/// `description_offre` passe par `html_to_plain` (structure en lignes
/// conservée) ; les champs courts sont ramenés à une seule ligne ; un champ
/// vide devient `None` ; un type de contrat hors du formulaire est écarté.
///
/// # Arguments
///
/// * `fields` - Champs bruts issus du JSON-LD, du LLM ou du texte collé.
///
/// # Returns
///
/// Les champs nettoyés.
pub fn finalize_fields(fields: ScrapedOfferFields) -> ScrapedOfferFields {
    ScrapedOfferFields {
        poste: fields.poste.as_deref().and_then(clean_single_line),
        entreprise: fields.entreprise.as_deref().and_then(clean_single_line),
        localisation: fields.localisation.as_deref().and_then(clean_single_line),
        type_contrat: fields
            .type_contrat
            .map(|t| t.trim().to_lowercase())
            .filter(|t| ALLOWED_CONTRACT_TYPES.contains(&t.as_str())),
        remuneration: fields.remuneration.as_deref().and_then(clean_single_line),
        description_offre: fields.description_offre.as_deref().and_then(clean_description),
    }
}

/// Convertit une description d'offre (HTML, Markdown ou texte) en texte brut.
///
/// # Arguments
///
/// * `raw` - Description brute.
///
/// # Returns
///
/// Le texte nettoyé, ou `None` s'il est vide.
pub fn clean_description(raw: &str) -> Option<String> {
    html_to_plain(raw)
}

/// Nettoie un champ court et le ramène à une ligne.
fn clean_single_line(raw: &str) -> Option<String> {
    let plain = html_to_plain(raw)?;
    let line = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    // Une puce isolée (« - Paris ») n'a pas de sens pour un champ court.
    let line = line.strip_prefix("- ").unwrap_or(&line).to_string();
    (!line.is_empty()).then_some(line)
}
