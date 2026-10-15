//! Rédaction des champs de prose du CV (titre, résumé, groupes de
//! compétences) via l'API Claude, avec revue et retry
//! (`llm::writing_loop`). Tout le reste du CV est rempli par Rust, dont les
//! expériences, projets et formations, repris du profil sans reformulation. Les
//! consignes de la référence (zones `[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`
//! du titre et du résumé) sont transmises au LLM et vérifiées.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::budgets::CvBudget;
use crate::docgen::example_facts::cv_information_values;
use crate::docgen::lint::{lint_cv_budget, Violation};
use crate::docgen::selection::Selection;
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::profile_facts::profile_facts_block;
use crate::llm::prose_checks::ProseChecks;
use crate::llm::writing_loop::{write_with_retries, Review, WritingFeedback, WritingOutcome};
use crate::llm::writing_rules::{guidance_block, language_rule, style_model_block, with_feedback, COMMON_RULES};
use crate::models::{CvContent, CvData, MatchingResult, ProfileDto, ProseGuidance};
use crate::llm::target::ClaudeTarget;

/// Entrées de la rédaction du CV.
///
/// # Attributes
///
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `matching` - Résultat du matching.
/// * `selection` - Entrées du profil retenues.
/// * `profile` - Profil complet du candidat.
/// * `example` - Contenu de la référence (modèle de style).
/// * `guidance` - Consignes de la référence (champs aux chemins de
///   `CvData` : `identity.title`, `summary`).
/// * `budget` - Budget de longueur.
/// * `target_lang` - Langue du CV (celle de l'offre).
/// * `example_lang` - Langue de la référence.
pub struct CvWritingInput<'a> {
    pub offer_analysis: &'a str,
    pub matching: &'a MatchingResult,
    pub selection: &'a Selection,
    pub profile: &'a ProfileDto,
    pub example: &'a CvData,
    pub guidance: &'a ProseGuidance,
    pub budget: CvBudget,
    pub target_lang: &'a str,
    pub example_lang: &'a str,
}

/// Schéma d'une chaîne non vide.
fn text() -> Value {
    json!({ "type": "string", "minLength": 1 })
}

/// Schéma d'une liste non vide.
fn list(items: Value) -> Value {
    json!({ "type": "array", "items": items, "minItems": 1 })
}

/// Schéma de sortie de la rédaction du CV.
fn cv_content_schema() -> Value {
    object(&[
        ("title", text()),
        ("summary", text()),
        ("skills_groups", list(object(&[("category", text()), ("skills", list(text()))]))),
    ])
}

/// Consignes de la référence ramenées aux chemins du contenu rédigé : le
/// titre est `identity.title` dans les données, `title` dans la rédaction.
fn content_guidance(guidance: &ProseGuidance) -> ProseGuidance {
    let mut mapped = guidance.clone();
    for zone in mapped.zones.iter_mut().filter(|zone| zone.field == "identity.title") {
        zone.field = "title".to_string();
    }
    mapped
}

/// Prose de la référence : modèle de style et base des formules autorisées.
pub fn example_prose(example: &CvData) -> String {
    let categories = example.skills_groups.iter().map(|g| g.category.as_str()).collect::<Vec<_>>().join(" ; ");
    format!(
        "Titre : {}\nRésumé : {}\nCatégories de compétences : {categories}",
        example.identity.title, example.summary,
    )
}

/// Bloc des budgets du CV.
fn budget_block(input: &CvWritingInput<'_>) -> String {
    let b = &input.budget;
    let mut lines = vec![
        "BUDGETS (limites dures)".to_string(),
        format!("summary : entre {} et {} mots.", b.summary_words.min, b.summary_words.max),
    ];
    lines.push(format!(
        "skills_groups : exactement {} groupes. Chaque compétence est recopiée telle qu'elle est écrite dans la liste des compétences du profil, sans en ajouter.",
        b.skills_group_count
    ));
    lines.join("\n")
}

/// Prompt système de la rédaction du CV.
fn system_prompt(input: &CvWritingInput<'_>) -> String {
    format!(
        "Tu rédiges les champs de prose d'un CV adapté à une offre : le titre, le résumé et le regroupement des compétences. Les expériences, projets, formations et langues sont ajoutés à part, tels qu'ils sont écrits dans le profil : ne les rédige pas.\n\n\
         {COMMON_RULES}\n\
         {}\n\n\
         Description des champs.\n\
         title : le titre professionnel affiché sous le nom, adapté à l'offre.\n\
         summary : le résumé de la section profil.\n\
         skills_groups : les compétences du profil regroupées en catégories pertinentes pour l'offre.",
        language_rule(input.target_lang, input.example_lang)
    )
}

/// Message utilisateur de la rédaction du CV.
fn user_message(input: &CvWritingInput<'_>) -> String {
    let mut blocks = vec![
        format!("ANALYSE DE L'OFFRE :\n{}", input.offer_analysis),
        profile_facts_block(input.profile, input.selection, input.matching),
        style_model_block(&example_prose(input.example)),
    ];
    blocks.extend(guidance_block(&content_guidance(input.guidance)));
    blocks.push(budget_block(input));
    blocks.join("\n\n")
}

/// Validation bloquante : champs de prose vides. Un document ne peut pas
/// être assemblé sans eux.
fn blocking_problems(content: &CvContent) -> Vec<String> {
    let empty = content.prose_fields().into_iter().filter(|f| f.text.trim().is_empty());
    empty.map(|f| format!("{} est vide.", f.path)).collect()
}

/// Compétences du profil, normalisées pour la comparaison.
fn known_skills(profile: &ProfileDto) -> Vec<String> {
    profile.skills.iter().map(|s| s.nom.trim().to_lowercase()).collect()
}

/// Vérifie que chaque compétence existe dans le profil et n'apparaît
/// qu'une fois. Non bloquant : après le dernier essai, les compétences en
/// cause sont retirées (`retain_profile_skills`), ce qui ne peut rien inventer.
fn skill_violations(content: &CvContent, profile: &ProfileDto) -> Vec<Violation> {
    let known = known_skills(profile);
    let mut seen: Vec<String> = Vec::new();
    let mut violations = Vec::new();
    for group in &content.skills_groups {
        for skill in &group.skills {
            let normalized = skill.trim().to_lowercase();
            if !known.contains(&normalized) {
                violations.push(Violation {
                    field: "skills_groups".to_string(),
                    message: format!("« {skill} » n'est pas une compétence du profil. N'utilise que les compétences listées, écrites à l'identique."),
                });
            } else if seen.contains(&normalized) {
                violations.push(Violation {
                    field: "skills_groups".to_string(),
                    message: format!("« {skill} » apparaît dans plusieurs groupes. Chaque compétence n'apparaît qu'une fois."),
                });
            }
            seen.push(normalized);
        }
    }
    violations
}

/// Retire les compétences vides, absentes du profil ou en double, puis les
/// groupes devenus vides.
fn retain_profile_skills(content: &mut CvContent, profile: &ProfileDto) {
    let known = known_skills(profile);
    let mut seen: Vec<String> = Vec::new();
    for group in content.skills_groups.iter_mut() {
        group.skills.retain(|skill| {
            let normalized = skill.trim().to_lowercase();
            let keep = known.contains(&normalized) && !seen.contains(&normalized);
            seen.push(normalized);
            keep
        });
    }
    content.skills_groups.retain(|g| !g.category.trim().is_empty() && !g.skills.is_empty());
}

/// Retire les compétences vides : le schéma demande des chaînes non vides,
/// mais la contrainte n'est pas toujours appliquée par l'API.
fn drop_empty_skills(content: &mut CvContent) {
    for group in content.skills_groups.iter_mut() {
        group.skills.retain(|skill| !skill.trim().is_empty());
    }
}

/// Revue complète d'un contenu de CV.
fn review(content: &CvContent, input: &CvWritingInput<'_>, checks: &ProseChecks) -> Review {
    let mut violations = checks.violations(&content.prose_fields());
    violations.extend(lint_cv_budget(content, &input.budget));
    violations.extend(skill_violations(content, input.profile));
    Review { blocking: blocking_problems(content), violations }
}

/// Appelle le LLM une fois.
async fn call_once(
    input: &CvWritingInput<'_>,
    feedback: Option<WritingFeedback>,
    target: &ClaudeTarget,
) -> Result<CvContent, AppError> {
    let message = with_feedback(user_message(input), feedback.as_ref());
    let raw = call_claude(&system_prompt(input), &message, Some(cv_content_schema()), target).await?;
    let mut content: CvContent = serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("La rédaction du CV n'a pas retourné le JSON attendu : {e}\nRéponse brute : {raw}"))
    })?;
    drop_empty_skills(&mut content);
    Ok(content)
}

/// Rédige les champs de prose du CV, avec revue et jusqu'à 2 retries.
///
/// # Arguments
///
/// * `input` - Entrées de la rédaction.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `WritingOutcome<CvContent>`.
pub async fn write_cv(input: &CvWritingInput<'_>, target: &ClaudeTarget) -> Result<WritingOutcome<CvContent>, AppError> {
    let checks = ProseChecks {
        example_prose: example_prose(input.example),
        guidance: content_guidance(input.guidance),
        information: cv_information_values(input.example),
    };
    write_with_retries(
        "CV",
        |feedback| call_once(input, feedback, target),
        |content| review(content, input, &checks),
        |content| {
            content.map_prose(|path, text| checks.clean(path, text));
            retain_profile_skills(content, input.profile);
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CvExperience, CvProject, ProjetView, SkillView, SkillsGroup, UserView};

    /// Profil fictif minimal : 2 projets, 2 compétences, aucune expérience.
    fn profile() -> ProfileDto {
        let now = chrono::Utc::now();
        let projet = |id: i64| ProjetView {
            id,
            titre: format!("Projet {id}"),
            description: None,
            stack: None,
            url: None,
            pour_qui: None,
            date_debut: None,
            date_fin: None,
        };
        ProfileDto {
            user: UserView {
                id: 1,
                nom: None,
                prenom: None,
                email: None,
                telephone: None,
                ville: None,
                zip_code: None,
                permis: None,
                description: None,
                linkedin: None,
                github: None,
                portfolio: None,
                interets: None,
                created_at: now,
            },
            experiences: Vec::new(),
            formations: Vec::new(),
            projets: vec![projet(1), projet(2)],
            skills: vec![
                SkillView { id: 1, domaine: "Langages".to_string(), nom: "Rust".to_string() },
                SkillView { id: 2, domaine: "Data".to_string(), nom: "SQL".to_string() },
            ],
            langues: Vec::new(),
        }
    }

    fn content(skills: &[&str]) -> CvContent {
        CvContent {
            title: "Titre".to_string(),
            summary: "Résumé".to_string(),
            skills_groups: vec![SkillsGroup {
                category: "Cat".to_string(),
                skills: skills.iter().map(|s| s.to_string()).collect(),
            }],
        }
    }

    #[test]
    fn schema_sans_experiences_ni_projets() {
        let schema = cv_content_schema();
        let mut keys: Vec<&String> = schema["properties"].as_object().expect("propriétés").keys().collect();
        keys.sort();
        assert_eq!(keys, vec!["skills_groups", "summary", "title"]);
    }

    #[test]
    fn champs_vides_et_competences_valides() {
        assert!(blocking_problems(&content(&["rust", "SQL"])).is_empty());
        assert!(skill_violations(&content(&["rust", "SQL"]), &profile()).is_empty());

        let empty = CvContent { title: " ".to_string(), summary: String::new(), ..content(&[]) };
        assert_eq!(blocking_problems(&empty), vec!["title est vide.".to_string(), "summary est vide.".to_string()]);

        let skills = skill_violations(&content(&["Kubernetes", "Rust", "rust"]), &profile());
        assert!(skills.iter().any(|v| v.message.contains("Kubernetes")), "{skills:?}");
        assert!(skills.iter().any(|v| v.message.contains("plusieurs groupes")), "{skills:?}");
    }

    #[test]
    fn competences_hors_profil_retirees_mecaniquement() {
        let mut cleaned = content(&["Kubernetes", "Rust", "rust", " "]);
        drop_empty_skills(&mut cleaned);
        retain_profile_skills(&mut cleaned, &profile());
        assert_eq!(cleaned.skills_groups[0].skills, vec!["Rust".to_string()]);

        let mut emptied = content(&["Kubernetes"]);
        retain_profile_skills(&mut emptied, &profile());
        assert!(emptied.skills_groups.is_empty(), "un groupe vidé disparaît");
    }

    #[test]
    fn modele_de_style_sans_bullets_de_la_reference() {
        let example = CvData {
            summary: "Résumé exemple".to_string(),
            experiences: vec![CvExperience { bullets: vec!["Bullet d'expérience".to_string()], ..Default::default() }],
            projects: vec![CvProject { bullets: vec!["Bullet de projet".to_string()], ..Default::default() }],
            ..Default::default()
        };
        let prose = example_prose(&example);
        assert!(prose.contains("Résumé exemple"), "{prose}");
        assert!(!prose.contains("Bullet"), "aucun contenu d'entrée de la référence : {prose}");
    }
}
