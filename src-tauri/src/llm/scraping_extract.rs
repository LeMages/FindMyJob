//! Extraction des champs structurés du formulaire de candidature à partir du
//! texte scrapé d'une page d'offre d'emploi.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude;

/// Champs du formulaire de candidature extraits d'une offre scrapée. Tous
/// optionnels : le scraping peut ne pas trouver certaines informations.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ScrapedOfferFields {
    pub poste: Option<String>,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub type_contrat: Option<String>,
    pub remuneration: Option<String>,
    pub description_offre: Option<String>,
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
            "description_offre": { "type": ["string", "null"] }
        },
        "required": [
            "poste", "entreprise", "localisation", "type_contrat",
            "remuneration", "description_offre"
        ],
        "additionalProperties": false
    })
}

/// Extrait les champs du formulaire de candidature depuis le texte scrapé
/// d'une offre d'emploi, via l'API Claude.
///
/// # Arguments
///
/// * `text` - Texte nettoyé de la page d'offre (voir `scraping::extract`).
///
/// # Returns
///
/// Les champs extraits, chacun `None` si non trouvé dans le texte. Toute
/// erreur d'appel, de refus ou de format inattendu remonte telle quelle.
pub async fn extract_offer_fields(text: &str) -> Result<ScrapedOfferFields, AppError> {
    let system_prompt = r#"Tu es un extracteur de données d'offres d'emploi. À partir du texte d'une page web d'offre d'emploi, extrais les informations suivantes en JSON.

Règles :
- "poste" : l'intitulé exact du poste tel qu'écrit dans l'offre
- "entreprise" : le nom de l'entreprise qui recrute
- "localisation" : la ville et/ou le pays du poste. Si télétravail, indique "Télétravail" ou "Télétravail + <ville>"
- "type_contrat" : DOIT être exactement une de ces valeurs : "stage", "alternance", "interim", "cdd", "cdi". Si le texte dit "internship" → "stage", "apprenticeship" → "alternance". Si indéterminé ou si aucune de ces valeurs ne correspond (ex. freelance), retourne null.
- "remuneration" : le salaire ou la gratification tel qu'indiqué (montant + période). Si absent, null.
- "description_offre" : le contenu complet de l'offre (missions, profil recherché, compétences, avantages). Conserver le texte tel quel, nettoyé et structuré en paragraphes lisibles. Ne pas résumer, garder le maximum d'informations.

Si une information n'est pas trouvable dans le texte, retourne null pour ce champ.
Réponds uniquement en JSON valide, sans markdown ni commentaires."#;

    let user_message = format!("Voici le texte de la page d'offre d'emploi :\n\n{text}");

    let raw_response =
        call_claude(system_prompt, &user_message, Some(scraped_offer_schema())).await?;

    // Le structured output garantit normalement un JSON conforme au schéma ;
    // ce parsing reste un filet de sécurité plutôt qu'une validation de forme.
    serde_json::from_str::<ScrapedOfferFields>(raw_response.trim()).map_err(|e| {
        AppError::Llm(format!(
            "L'extraction de l'offre n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
        ))
    })
}
