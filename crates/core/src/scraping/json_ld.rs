//! Étage A de la cascade d'extraction : lecture du `JobPosting` schema.org
//! publié en JSON-LD par la page d'offre, sans LLM.

// Importation des crates externes
use scraper::{Html, Selector};
use serde_json::Value;

// Importation des modules internes
use crate::llm::scraping_extract::ScrapedOfferFields;
use crate::scraping::contract_type::resolve_contract_type;
use crate::scraping::finalize::finalize_fields;
use crate::scraping::salary_format::format_base_salary;

/// Longueur minimale (en caractères) de la description nettoyée pour que
/// l'étage JSON-LD soit considéré comme réussi. En dessous, il s'agit d'un
/// résumé ou d'un extrait : l'étage LLM lira la page complète.
const MIN_DESCRIPTION_CHARS: usize = 200;

/// Profondeur maximale de recherche d'un `JobPosting` dans le JSON-LD
/// (tableaux, `@graph` imbriqués).
const MAX_SEARCH_DEPTH: usize = 8;

/// Extrait les champs d'une offre depuis le JSON-LD d'une page.
///
/// Lit tous les blocs `<script type="application/ld+json">`, cherche un
/// `JobPosting` (objet, tableau ou `@graph`, `@type` chaîne ou tableau) et
/// le convertit en champs nettoyés.
///
/// # Arguments
///
/// * `html` - HTML brut de la page.
///
/// # Returns
///
/// Les champs nettoyés si l'étage réussit (poste présent et description de
/// plus de [`MIN_DESCRIPTION_CHARS`] caractères), sinon `None`. Les champs
/// manquants restent à `None`.
pub fn extract_job_posting_fields(html: &str) -> Option<ScrapedOfferFields> {
    let posting = find_job_posting(html)?;
    let fields = finalize_fields(map_job_posting(&posting));

    let description_len = fields.description_offre.as_deref().map_or(0, |d| d.chars().count());
    (fields.poste.is_some() && description_len > MIN_DESCRIPTION_CHARS).then_some(fields)
}

/// Cherche le premier `JobPosting` parmi tous les blocs JSON-LD de la page.
///
/// # Arguments
///
/// * `html` - HTML brut de la page.
///
/// # Returns
///
/// L'objet `JobPosting`, ou `None` s'il n'y en a pas.
pub fn find_job_posting(html: &str) -> Option<Value> {
    let document = Html::parse_document(html);
    let selector =
        Selector::parse("script[type=\"application/ld+json\"]").expect("sélecteur CSS constant valide");
    document
        .select(&selector)
        .filter_map(|script| parse_lenient(&script.text().collect::<String>()))
        .find_map(|value| search_job_posting(&value, 0).cloned())
}

/// Parse un bloc JSON-LD. Certains sites y laissent des retours à la ligne
/// bruts dans les chaînes (JSON invalide) : second essai après les avoir
/// remplacés par des espaces.
fn parse_lenient(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok().or_else(|| {
        let flattened: String = trimmed
            .chars()
            .map(|c| if matches!(c, '\n' | '\r' | '\t') { ' ' } else { c })
            .collect();
        serde_json::from_str(&flattened).ok()
    })
}

/// Parcourt une valeur JSON-LD (objet, tableau, `@graph`) à la recherche
/// d'un objet dont le `@type` est ou contient `JobPosting`.
fn search_job_posting(value: &Value, depth: usize) -> Option<&Value> {
    if depth > MAX_SEARCH_DEPTH {
        return None;
    }
    match value {
        Value::Array(items) => items.iter().find_map(|item| search_job_posting(item, depth + 1)),
        Value::Object(map) => {
            if map.get("@type").is_some_and(is_job_posting_type) {
                return Some(value);
            }
            map.get("@graph").and_then(|graph| search_job_posting(graph, depth + 1))
        }
        _ => None,
    }
}

/// Indique si un `@type` (chaîne ou tableau) désigne un `JobPosting`.
fn is_job_posting_type(type_value: &Value) -> bool {
    match type_value {
        Value::String(s) => is_job_posting_name(s),
        Value::Array(items) => items.iter().filter_map(Value::as_str).any(is_job_posting_name),
        _ => false,
    }
}

/// Compare un nom de type, avec ou sans préfixe `schema:` / URL schema.org.
fn is_job_posting_name(name: &str) -> bool {
    name.rsplit(['/', ':']).next().is_some_and(|n| n == "JobPosting")
}

/// Convertit un `JobPosting` en champs bruts (avant nettoyage final).
///
/// # Arguments
///
/// * `posting` - Objet `JobPosting`.
///
/// # Returns
///
/// Les champs trouvés ; les absents restent à `None`.
pub fn map_job_posting(posting: &Value) -> ScrapedOfferFields {
    let poste = text_of(posting.get("title"));
    let type_contrat = resolve_contract_type(posting.get("employmentType"), poste.as_deref());
    ScrapedOfferFields {
        entreprise: posting.get("hiringOrganization").and_then(organization_name),
        localisation: format_location(posting),
        type_contrat: type_contrat.map(str::to_string),
        remuneration: posting.get("baseSalary").and_then(format_base_salary),
        description_offre: text_of(posting.get("description")),
        poste,
    }
}

/// Nom de l'entreprise : `hiringOrganization` objet (`name`) ou chaîne.
fn organization_name(organization: &Value) -> Option<String> {
    match organization {
        Value::Object(map) => text_of(map.get("name")),
        Value::Array(items) => items.iter().find_map(organization_name),
        other => text_of(Some(other)),
    }
}

/// Localisation lisible : lieux de `jobLocation` (objet ou tableau) joints
/// par « / », précédés de « Télétravail » si `jobLocationType` vaut
/// `TELECOMMUTE`.
fn format_location(posting: &Value) -> Option<String> {
    let mut places: Vec<String> = Vec::new();
    match posting.get("jobLocation") {
        Some(Value::Array(items)) => places.extend(items.iter().filter_map(place_name)),
        Some(location) => places.extend(place_name(location)),
        None => {}
    }
    places.dedup();
    let joined = (!places.is_empty()).then(|| places.join(" / "));

    let remote = posting.get("jobLocationType").is_some_and(is_telecommute);
    match (remote, joined) {
        (true, Some(places)) => Some(format!("Télétravail + {places}")),
        (true, None) => Some("Télétravail".to_string()),
        (false, joined) => joined,
    }
}

/// Nom d'un lieu `Place` : ville, sinon région, sinon pays de son adresse.
fn place_name(place: &Value) -> Option<String> {
    let address = match place {
        Value::Object(map) => map.get("address").unwrap_or(place),
        other => other,
    };
    match address {
        Value::Object(map) => ["addressLocality", "addressRegion", "addressCountry"]
            .iter()
            .find_map(|key| map.get(*key).and_then(country_or_text)),
        other => text_of(Some(other)),
    }
}

/// Texte d'un champ d'adresse ; `addressCountry` peut être un objet `Country`.
fn country_or_text(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => text_of(map.get("name")),
        other => text_of(Some(other)),
    }
}

/// Indique si `jobLocationType` (chaîne ou tableau) vaut `TELECOMMUTE`.
fn is_telecommute(value: &Value) -> bool {
    match value {
        Value::String(s) => s.trim().eq_ignore_ascii_case("TELECOMMUTE"),
        Value::Array(items) => items.iter().any(is_telecommute),
        _ => false,
    }
}

/// Chaîne non vide d'une valeur JSON (chaîne ou nombre).
fn text_of(value: Option<&Value>) -> Option<String> {
    let text = match value? {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}
