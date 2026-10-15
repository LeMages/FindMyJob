//! Extraction des champs structurés du formulaire de candidature à partir du
//! texte scrapé d'une page d'offre d'emploi.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::scraping::failure::ScrapingFailure;
use crate::scraping::page_hints::PageHints;
use crate::llm::target::ClaudeTarget;

/// Champs du formulaire de candidature extraits d'une offre scrapée. Tous
/// optionnels : le scraping peut ne pas trouver certaines informations.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScrapedOfferFields {
    pub poste: Option<String>,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub type_contrat: Option<String>,
    pub remuneration: Option<String>,
    pub description_offre: Option<String>,
    /// Personne à contacter nommée par l'offre, s'il y en a une.
    #[serde(default)]
    pub contact: Option<ScrapedContact>,
}

/// Personne à contacter nommée par une offre. Sert uniquement à pré-remplir
/// le formulaire : rien n'est enregistré avant sa validation.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScrapedContact {
    pub nom: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
}

/// Schéma JSON attendu pour l'extraction, utilisé comme structured output
/// (`output_config.format`). `type_contrat` est restreint aux valeurs du
/// `<select>` du formulaire (`stage`/`alternance`/`interim`/`cdd`/`cdi`) —
/// pas `freelance`, absent de ce `<select>` bien qu'autorisé en base.
fn scraped_offer_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "poste": { "type": ["string", "null"] },
            "entreprise": { "type": ["string", "null"] },
            "localisation": { "type": ["string", "null"] },
            "type_contrat": {
                "anyOf": [
                    { "type": "string", "enum": ["stage", "alternance", "interim", "cdd", "cdi"] },
                    { "type": "null" }
                ]
            },
            "remuneration": { "type": ["string", "null"] },
            "description_offre": { "type": ["string", "null"] },
            "contact": {
                "anyOf": [
                    {
                        "type": "object",
                        "properties": {
                            "nom": { "type": ["string", "null"] },
                            "role": { "type": ["string", "null"] },
                            "email": { "type": ["string", "null"] },
                            "telephone": { "type": ["string", "null"] }
                        },
                        "required": ["nom", "role", "email", "telephone"],
                        "additionalProperties": false
                    },
                    { "type": "null" }
                ]
            }
        },
        "required": [
            "poste", "entreprise", "localisation", "type_contrat",
            "remuneration", "description_offre", "contact"
        ],
        "additionalProperties": false
    })
}

/// Prompt système de l'extraction : la description est recopiée depuis la
/// page, jamais résumée ni reformulée.
const SYSTEM_PROMPT: &str = r#"Tu es un extracteur de données d'offres d'emploi. À partir du texte d'une page web d'offre d'emploi, extrais les informations suivantes en JSON.

Règles :
- "poste" : l'intitulé exact du poste tel qu'écrit dans l'offre
- "entreprise" : le nom de l'entreprise qui recrute
- "localisation" : la ville et/ou le pays du poste. Si télétravail, indique "Télétravail" ou "Télétravail + <ville>"
- "type_contrat" : DOIT être exactement une de ces valeurs : "stage", "alternance", "interim", "cdd", "cdi". Si le texte dit "internship" → "stage", "apprenticeship" → "alternance". Si indéterminé ou si aucune de ces valeurs ne correspond (ex. freelance), retourne null.
- "remuneration" : le salaire ou la gratification tel qu'indiqué (montant + période). Si absent, null.
- "description_offre" : le contenu de l'offre (présentation, missions, profil recherché, compétences, avantages), RECOPIÉ MOT POUR MOT depuis la page. Ne résume pas, ne reformule pas, ne traduis pas, n'ajoute rien. Garde les titres de section de la page tels qu'ils sont écrits et les listes sous forme de lignes commençant par "- ". Exclus seulement ce qui n'appartient pas à l'offre (menus, bandeaux cookies, offres similaires, pied de page).
- "contact" : la personne à contacter pour cette candidature, UNIQUEMENT si l'offre nomme explicitement une personne (prénom et/ou nom). "nom" : son nom tel qu'écrit, sans civilité ajoutée ; "role" : sa fonction si elle est écrite ; "email" et "telephone" : ses coordonnées si elles sont écrites dans l'offre. Si l'offre ne nomme aucune personne (adresse générique, service RH, formulaire de candidature...), retourne null. N'invente jamais un nom, une fonction ou une coordonnée.

Les indices de titre fournis (balise <title>, og:title) peuvent aider à identifier le poste et l'entreprise, mais le texte de la page fait foi.
Si une information n'est pas trouvable dans le texte, retourne null pour ce champ.
Réponds uniquement en JSON valide, sans markdown ni commentaires."#;

/// Extrait les champs du formulaire de candidature depuis le texte d'une
/// offre d'emploi (page ou texte collé), via l'API Claude.
///
/// # Arguments
///
/// * `text` - Texte de l'offre (voir `scraping::extract`, ou texte collé).
/// * `hints` - Indices de titre de la page (vides pour un texte collé).
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// Les champs extraits, bruts (le nettoyage final est fait par
/// `scraping::finalize`). `AppError::Scraping(InsufficientContent)` si le
/// LLM ne trouve ni poste ni description ; toute erreur d'appel, de refus ou
/// de format inattendu remonte telle quelle.
pub async fn extract_offer_fields(
    text: &str,
    hints: &PageHints,
    target: &ClaudeTarget,
) -> Result<ScrapedOfferFields, AppError> {
    let user_message = build_user_message(text, hints);
    let raw_response = call_claude(SYSTEM_PROMPT, &user_message, Some(scraped_offer_schema()), target).await?;

    // Le structured output garantit normalement un JSON conforme au schéma ;
    // ce parsing reste un filet de sécurité plutôt qu'une validation de forme.
    let fields = serde_json::from_str::<ScrapedOfferFields>(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "L'extraction de l'offre n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })?;

    if is_blank(fields.poste.as_deref()) || is_blank(fields.description_offre.as_deref()) {
        return Err(AppError::Scraping(ScrapingFailure::InsufficientContent));
    }
    Ok(fields)
}

/// Assemble le message utilisateur : indices de titre éventuels, puis texte.
fn build_user_message(text: &str, hints: &PageHints) -> String {
    let mut message = String::new();
    if !hints.is_empty() {
        message.push_str("Indices de titre de la page :\n");
        if let Some(title) = &hints.title {
            message.push_str(&format!("- <title> : {title}\n"));
        }
        if let Some(og_title) = &hints.og_title {
            message.push_str(&format!("- og:title : {og_title}\n"));
        }
        message.push('\n');
    }
    message.push_str(&format!("Voici le texte de la page d'offre d'emploi :\n\n{text}"));
    message
}

/// Indique si un champ est absent ou vide.
fn is_blank(value: Option<&str>) -> bool {
    value.is_none_or(|v| v.trim().is_empty())
}
