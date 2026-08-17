//! Rédaction du contenu du CV et de la lettre de motivation via l'API
//! Claude — étapes 3a/3b du pipeline de génération de documents. Produit du
//! texte structuré (pas de Typst) : c'est `llm::assemble` qui le transforme
//! ensuite en document.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::{call_claude, call_claude_with_tokens_meta};
use crate::models::ProfileDto;

/// Règles communes aux deux rédactions, rappelées dans chaque prompt système
/// pour garantir l'honnêteté du contenu généré.
const RULES_HONNETETE: &str = "Règles strictes :\n\
- Tu ne peux que reformuler ce qui existe dans le profil, JAMAIS inventer une compétence, une expérience ou un projet\n\
- Adapte le wording pour reprendre les mots-clés de l'offre quand c'est honnête\n\
- Rédige dans la langue de l'offre (champ `language` de l'analyse d'offre)";

/// Schéma JSON attendu pour le contenu du CV.
fn cv_content_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "header": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" }
                },
                "required": ["title"],
                "additionalProperties": false
            },
            "summary": { "type": "string" },
            "experiences": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "bullets": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "bullets"],
                    "additionalProperties": false
                }
            },
            "projects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "number" },
                        "description": { "type": "string" }
                    },
                    "required": ["id", "description"],
                    "additionalProperties": false
                }
            },
            "skills_grouping": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "category": { "type": "string" },
                        "skills": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["category", "skills"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["header", "summary", "experiences", "projects", "skills_grouping"],
        "additionalProperties": false
    })
}

/// Vrai si la valeur est absente, n'est pas une string, ou ne contient que
/// des espaces — utilisé par `validate_cv_content` et `validate_lm_content`
/// pour détecter les champs "vides mais conformes au schéma".
fn is_blank(value: Option<&serde_json::Value>) -> bool {
    value.and_then(|v| v.as_str()).map(str::trim).unwrap_or("").is_empty()
}

/// Vérifie que le contenu du CV parsé n'a pas de champ vide malgré un JSON
/// structurellement conforme au schéma (cas observé : le modèle retourne
/// `""`/`[]` partout tout en respectant `cv_content_schema`).
///
/// # Arguments
///
/// * `parsed` - JSON du CV déjà parsé (voir `generate_cv_content`).
///
/// # Returns
///
/// `Ok(())` si tous les champs obligatoires ont du contenu substantiel,
/// sinon `Err` avec un message décrivant le premier champ vide rencontré.
fn validate_cv_content(parsed: &serde_json::Value) -> Result<(), String> {
    let obj = parsed.as_object().ok_or_else(|| "le JSON racine n'est pas un objet".to_string())?;

    if is_blank(obj.get("header").and_then(|h| h.get("title"))) {
        return Err("header.title est vide".to_string());
    }
    if is_blank(obj.get("summary")) {
        return Err("summary est vide".to_string());
    }

    let experiences = obj.get("experiences").and_then(|v| v.as_array());
    let Some(experiences) = experiences.filter(|e| !e.is_empty()) else {
        return Err("experiences est vide".to_string());
    };
    for (i, experience) in experiences.iter().enumerate() {
        let bullets = experience.get("bullets").and_then(|v| v.as_array());
        let Some(bullets) = bullets.filter(|b| !b.is_empty()) else {
            return Err(format!("experiences[{i}].bullets est vide"));
        };
        if bullets.iter().any(|b| is_blank(Some(b))) {
            return Err(format!("experiences[{i}].bullets contient un bullet vide"));
        }
    }

    let projects = obj.get("projects").and_then(|v| v.as_array());
    let Some(projects) = projects.filter(|p| !p.is_empty()) else {
        return Err("projects est vide".to_string());
    };
    for (i, project) in projects.iter().enumerate() {
        if is_blank(project.get("description")) {
            return Err(format!("projects[{i}].description est vide"));
        }
    }

    let skills_grouping = obj.get("skills_grouping").and_then(|v| v.as_array());
    let Some(skills_grouping) = skills_grouping.filter(|s| !s.is_empty()) else {
        return Err("skills_grouping est vide".to_string());
    };
    for (i, group) in skills_grouping.iter().enumerate() {
        if is_blank(group.get("category")) {
            return Err(format!("skills_grouping[{i}].category est vide"));
        }
        let skills = group.get("skills").and_then(|v| v.as_array());
        if skills.filter(|s| !s.is_empty()).is_none() {
            return Err(format!("skills_grouping[{i}].skills est vide"));
        }
    }

    Ok(())
}

/// Schéma JSON attendu pour le contenu de la lettre de motivation.
fn lm_content_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "opening": { "type": "string" },
            "body_paragraphs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "argument": { "type": "string" },
                        "evidence": { "type": "string" }
                    },
                    "required": ["argument", "evidence"],
                    "additionalProperties": false
                }
            },
            "closing": { "type": "string" },
            "tone_notes": { "type": "string" }
        },
        "required": ["opening", "body_paragraphs", "closing", "tone_notes"],
        "additionalProperties": false
    })
}

/// Vérifie que le contenu de la lettre parsé n'a pas de champ vide malgré un
/// JSON structurellement conforme au schéma (même défaut que pour le CV, voir
/// `validate_cv_content`).
///
/// # Arguments
///
/// * `parsed` - JSON de la lettre déjà parsé (voir `generate_lm_content`).
///
/// # Returns
///
/// `Ok(())` si tous les champs obligatoires ont du contenu substantiel,
/// sinon `Err` avec un message décrivant le premier champ vide rencontré.
fn validate_lm_content(parsed: &serde_json::Value) -> Result<(), String> {
    let obj = parsed.as_object().ok_or_else(|| "le JSON racine n'est pas un objet".to_string())?;

    if is_blank(obj.get("opening")) {
        return Err("opening est vide".to_string());
    }

    let body_paragraphs = obj.get("body_paragraphs").and_then(|v| v.as_array());
    let Some(body_paragraphs) = body_paragraphs.filter(|p| !p.is_empty()) else {
        return Err("body_paragraphs est vide".to_string());
    };
    for (i, paragraph) in body_paragraphs.iter().enumerate() {
        if is_blank(paragraph.get("argument")) {
            return Err(format!("body_paragraphs[{i}].argument est vide"));
        }
        if is_blank(paragraph.get("evidence")) {
            return Err(format!("body_paragraphs[{i}].evidence est vide"));
        }
    }

    if is_blank(obj.get("closing")) {
        return Err("closing est vide".to_string());
    }
    if is_blank(obj.get("tone_notes")) {
        return Err("tone_notes est vide".to_string());
    }

    Ok(())
}

/// Rédige le contenu textuel d'un CV adapté à l'offre, à partir de la
/// stratégie de matching et du profil du candidat.
///
/// # Arguments
///
/// * `matching` - JSON produit par `llm::matching::match_profile`.
/// * `profile` - Profil complet du candidat.
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Le JSON de contenu du CV (sérialisé de façon canonique).
pub async fn generate_cv_content(
    matching: &str,
    profile: &ProfileDto,
    offer_analysis: &str,
    model: &str,
) -> Result<String, AppError> {
    let system_prompt = format!(
        "Tu rédiges le contenu d'un CV adapté à l'offre.\n\n{RULES_HONNETETE}\n\
- Les éléments à faible pertinence (`relevance: low`) peuvent être raccourcis mais pas supprimés\n\
- Tous les champs sont obligatoires et doivent contenir du contenu substantiel. Aucun champ string ne doit être vide, aucun tableau ne doit être vide.\n\n\
Description des champs à renseigner :\n\
- header.title : titre professionnel adapté à l'offre\n\
- summary : résumé/accroche en 3-4 lignes, adapté à l'offre\n\
- experiences : pour chaque expérience du profil, ses bullet points reformulés (repris de la description originale, jamais inventés)\n\
- projects : pour chaque projet du profil, sa description reformulée\n\
- skills_grouping : compétences du profil regroupées par catégorie pertinente pour l'offre"
    );

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let user_message = format!(
        "Analyse d'offre :\n{offer_analysis}\n\nStratégie de matching :\n{matching}\n\nProfil du candidat :\n{profile_json}"
    );

    // DEBUG: à supprimer
    eprintln!(
        "[CV-INPUT] === SYSTEM PROMPT ({} chars) ===\n{}",
        system_prompt.len(),
        system_prompt
    );
    // DEBUG: à supprimer
    eprintln!(
        "[CV-INPUT] === USER MESSAGE ({} chars) ===\n{}",
        user_message.len(),
        user_message
    );

    // On tolère un seul échec de validation (JSON conforme au schéma mais
    // vide) : le modèle produit parfois un contenu creux au premier essai et
    // se corrige au second. Une erreur de parsing JSON, elle, n'est jamais
    // retentée — c'est un problème de format, pas de contenu.
    let mut validation_error: Option<String> = None;
    for attempt in 1..=2u8 {
        let raw_response = call_claude(&system_prompt, &user_message, Some(cv_content_schema()), model).await?;

        // DEBUG: à supprimer
        eprintln!(
            "[CV-RAW] === RESPONSE ({} chars, tentative {attempt}/2) ===\n{}",
            raw_response.len(),
            raw_response
        );

        let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
            AppError::Llm(format!(
                "La rédaction du CV n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
            ))
        })?;

        match validate_cv_content(&parsed) {
            Ok(()) => {
                return serde_json::to_string(&parsed)
                    .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")));
            }
            Err(e) => {
                eprintln!("[llm::content] CV incomplet (tentative {attempt}/2) : {e}");
                validation_error = Some(e);
            }
        }
    }

    Err(AppError::Llm(format!(
        "Le contenu du CV généré est vide ou incomplet après 2 tentatives : {}",
        validation_error.unwrap_or_default()
    )))
}

/// Rédige le contenu textuel d'une lettre de motivation adaptée à l'offre, à
/// partir de la stratégie de matching et du profil du candidat.
///
/// # Arguments
///
/// * `matching` - JSON produit par `llm::matching::match_profile`.
/// * `profile` - Profil complet du candidat.
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// Le JSON de contenu de la lettre (sérialisé de façon canonique).
pub async fn generate_lm_content(
    matching: &str,
    profile: &ProfileDto,
    offer_analysis: &str,
    model: &str,
) -> Result<String, AppError> {
    let system_prompt = format!(
        "Tu rédiges le contenu d'une lettre de motivation.\n\n{RULES_HONNETETE}\n\
- Structure : accroche, 2-3 paragraphes argumentatifs, conclusion avec appel à l'action\n\
- Chaque argument doit s'appuyer sur une expérience ou un projet concret du profil\n\
- Tous les champs sont obligatoires et doivent contenir du contenu substantiel. Aucun champ string ne doit être vide, aucun tableau ne doit être vide.\n\n\
Description des champs à renseigner :\n\
- opening : accroche — pourquoi ce poste, cette entreprise\n\
- body_paragraphs : pour chaque paragraphe, l'argument principal et l'expérience/le projet qui le soutient\n\
- closing : conclusion et appel à l'action\n\
- tone_notes : ton adopté et pourquoi"
    );

    let profile_json = serde_json::to_string(profile)
        .map_err(|e| AppError::Llm(format!("Erreur de sérialisation du profil: {e}")))?;

    let user_message = format!(
        "Analyse d'offre :\n{offer_analysis}\n\nStratégie de matching :\n{matching}\n\nProfil du candidat :\n{profile_json}"
    );

    // DEBUG: à supprimer
    eprintln!(
        "[LM-INPUT] === SYSTEM PROMPT ({} chars) ===\n{}",
        system_prompt.len(),
        system_prompt
    );
    // DEBUG: à supprimer
    eprintln!(
        "[LM-INPUT] === USER MESSAGE ({} chars) ===\n{}",
        user_message.len(),
        user_message
    );

    // On tolère un seul échec de validation (JSON conforme au schéma mais
    // vide) : le modèle produit parfois un contenu creux au premier essai et
    // se corrige au second. Une erreur de parsing JSON, elle, n'est jamais
    // retentée — c'est un problème de format, pas de contenu.
    let mut validation_error: Option<String> = None;
    for attempt in 1..=2u8 {
        let (raw_response, stop_reason) = call_claude_with_tokens_meta(
            &system_prompt,
            &user_message,
            Some(lm_content_schema()),
            crate::llm::client::MAX_TOKENS,
            model,
        )
        .await?;

        // DEBUG: à supprimer
        eprintln!(
            "[LM-RAW] === RESPONSE ({} chars, tentative {attempt}/2) ===\n{}\n[LM-RAW] stop_reason={stop_reason:?}",
            raw_response.len(),
            raw_response
        );

        let parsed: serde_json::Value = serde_json::from_str(raw_response.trim()).map_err(|e| {
            AppError::Llm(format!(
                "La rédaction de la lettre n'a pas retourné du JSON valide: {e}\nRéponse brute: {raw_response}"
            ))
        })?;

        match validate_lm_content(&parsed) {
            Ok(()) => {
                return serde_json::to_string(&parsed)
                    .map_err(|e| AppError::Llm(format!("Erreur de re-sérialisation: {e}")));
            }
            Err(e) => {
                eprintln!("[llm::content] Lettre incomplète (tentative {attempt}/2) : {e}");
                validation_error = Some(e);
            }
        }
    }

    Err(AppError::Llm(format!(
        "Le contenu de la lettre généré est vide ou incomplet après 2 tentatives : {}",
        validation_error.unwrap_or_default()
    )))
}
