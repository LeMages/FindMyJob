//! Rédaction des champs de prose du CV (titre, résumé, bullets, groupes de
//! compétences) via l'API Claude, avec revue et retry
//! (`llm::writing_loop`). Tout le reste du CV est rempli par Rust.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::docgen::budgets::CvBudget;
use crate::docgen::lint::{lint_cv_budget, lint_prose, strip_dashes, Violation};
use crate::docgen::selection::Selection;
use crate::error::AppError;
use crate::llm::client::call_claude;
use crate::llm::doc_data_schema::object;
use crate::llm::profile_facts::profile_facts_block;
use crate::llm::writing_loop::{write_with_retries, Review, WritingFeedback, WritingOutcome};
use crate::llm::writing_rules::{language_rule, style_model_block, with_feedback, COMMON_RULES};
use crate::models::{CvContent, CvData, ItemBullets, MatchingResult, ProfileDto};

/// Entrées de la rédaction du CV.
///
/// # Attributes
///
/// * `offer_analysis` - JSON produit par `llm::offer::analyze_offer`.
/// * `matching` - Résultat du matching.
/// * `selection` - Entrées du profil retenues.
/// * `profile` - Profil complet du candidat.
/// * `example` - Contenu de la référence (modèle de style).
/// * `budget` - Budget de longueur.
/// * `target_lang` - Langue du CV (celle de l'offre).
/// * `example_lang` - Langue de la référence.
pub struct CvWritingInput<'a> {
    pub offer_analysis: &'a str,
    pub matching: &'a MatchingResult,
    pub selection: &'a Selection,
    pub profile: &'a ProfileDto,
    pub example: &'a CvData,
    pub budget: CvBudget,
    pub target_lang: &'a str,
    pub example_lang: &'a str,
}

/// Schéma d'une chaîne non vide.
fn text() -> Value {
    json!({ "type": "string", "minLength": 1 })
}

/// Schéma d'une liste, non vide si `non_empty`.
fn list(items: Value, non_empty: bool) -> Value {
    if non_empty {
        json!({ "type": "array", "items": items, "minItems": 1 })
    } else {
        json!({ "type": "array", "items": items })
    }
}

/// Schéma de sortie de la rédaction du CV.
fn cv_content_schema(input: &CvWritingInput<'_>) -> Value {
    let entry = object(&[("id", json!({ "type": "integer" })), ("bullets", list(text(), true))]);
    object(&[
        ("title", text()),
        ("summary", text()),
        ("experiences", list(entry.clone(), !input.selection.experiences.is_empty())),
        ("projects", list(entry, !input.selection.projects.is_empty())),
        ("skills_groups", list(object(&[("category", text()), ("skills", list(text(), true))]), true)),
    ])
}

/// Prose de la référence : modèle de style et base des formules autorisées.
pub fn example_prose(example: &CvData) -> String {
    let bullets = |items: Vec<&Vec<String>>| {
        items.into_iter().flatten().map(|b| format!("- {b}")).collect::<Vec<_>>().join("\n")
    };
    let categories = example.skills_groups.iter().map(|g| g.category.as_str()).collect::<Vec<_>>().join(" ; ");
    format!(
        "Titre : {}\nRésumé : {}\nBullets d'expérience :\n{}\nBullets de projet :\n{}\nCatégories de compétences : {categories}",
        example.title,
        example.summary,
        bullets(example.experiences.iter().map(|e| &e.bullets).collect()),
        bullets(example.projects.iter().map(|p| &p.bullets).collect()),
    )
}

/// Bloc des budgets du CV, entrée par entrée.
fn budget_block(input: &CvWritingInput<'_>) -> String {
    let b = &input.budget;
    let mut lines = vec![
        "BUDGETS (limites dures)".to_string(),
        format!("summary : entre {} et {} mots.", b.summary_words.min, b.summary_words.max),
    ];
    for (rank, selected) in input.selection.experiences.iter().enumerate() {
        lines.push(format!("Expérience id {} : exactement {} bullets.", selected.item.id, b.experience_bullets_at(rank)));
    }
    for (rank, selected) in input.selection.projects.iter().enumerate() {
        lines.push(format!("Projet id {} : exactement {} bullets.", selected.item.id, b.project_bullets_at(rank)));
    }
    lines.push(format!(
        "Chaque bullet : au plus {} mots, environ {} en moyenne.",
        b.bullet_words.max, b.bullet_words.target
    ));
    lines.push(format!(
        "skills_groups : exactement {} groupes. Chaque compétence est recopiée telle qu'elle est écrite dans la liste des compétences du profil, sans en ajouter.",
        b.skills_group_count
    ));
    lines.join("\n")
}

/// Prompt système de la rédaction du CV.
fn system_prompt(input: &CvWritingInput<'_>) -> String {
    format!(
        "Tu rédiges les champs de prose d'un CV adapté à une offre : le titre, le résumé, les bullets de chaque expérience et de chaque projet retenus, et le regroupement des compétences. Les intitulés, entreprises, dates, formations et langues sont ajoutés à part : ne les rédige pas.\n\n\
         {COMMON_RULES}\n\
         {}\n\n\
         Description des champs.\n\
         title : le titre professionnel affiché sous le nom, adapté à l'offre.\n\
         summary : le résumé de la section profil.\n\
         experiences : pour chaque expérience retenue, son id et ses bullets.\n\
         projects : pour chaque projet retenu, son id et ses bullets.\n\
         skills_groups : les compétences du profil regroupées en catégories pertinentes pour l'offre.",
        language_rule(input.target_lang, input.example_lang)
    )
}

/// Message utilisateur de la rédaction du CV.
fn user_message(input: &CvWritingInput<'_>) -> String {
    format!(
        "ANALYSE DE L'OFFRE :\n{}\n\n{}\n\n{}\n\n{}",
        input.offer_analysis,
        profile_facts_block(input.profile, input.selection, input.matching),
        style_model_block(&example_prose(input.example)),
        budget_block(input)
    )
}

/// Remet les entrées du contenu dans l'ordre de la sélection (ordre
/// d'affichage), pour que les budgets par rang s'appliquent aux bonnes
/// entrées. Les identifiants inconnus sont placés à la fin.
pub fn order_by_selection(content: &CvContent, selection: &Selection) -> CvContent {
    let order = |items: &[ItemBullets], ids: &[i64]| {
        let mut sorted = items.to_vec();
        sorted.sort_by_key(|item| ids.iter().position(|id| *id == item.id).unwrap_or(usize::MAX));
        sorted
    };
    CvContent {
        experiences: order(&content.experiences, &selection.experience_ids()),
        projects: order(&content.projects, &selection.project_ids()),
        ..content.clone()
    }
}

/// Vérifie qu'une liste d'entrées couvre exactement les identifiants attendus.
fn check_ids(kind: &str, items: &[ItemBullets], expected: &[i64]) -> Vec<String> {
    let mut problems = Vec::new();
    for item in items {
        if !expected.contains(&item.id) {
            problems.push(format!("{kind} : l'id {} n'est pas une entrée retenue (ids attendus : {expected:?}).", item.id));
        }
    }
    for id in expected {
        match items.iter().filter(|item| item.id == *id).count() {
            0 => problems.push(format!("{kind} : l'id {id} manque.")),
            1 => {}
            n => problems.push(format!("{kind} : l'id {id} apparaît {n} fois.")),
        }
    }
    problems
}

/// Validation bloquante : identifiants et champs de prose vides. Un
/// document ne peut pas être assemblé sans eux.
fn blocking_problems(content: &CvContent, input: &CvWritingInput<'_>) -> Vec<String> {
    let mut problems = check_ids("experiences", &content.experiences, &input.selection.experience_ids());
    problems.extend(check_ids("projects", &content.projects, &input.selection.project_ids()));
    let empty = content.prose_fields().into_iter().filter(|f| f.text.trim().is_empty());
    problems.extend(empty.map(|f| format!("{} est vide.", f.path)));
    problems
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
fn review(content: &CvContent, input: &CvWritingInput<'_>, example_prose: &str) -> Review {
    let ordered = order_by_selection(content, input.selection);
    let mut violations = lint_prose(&ordered.prose_fields(), example_prose);
    violations.extend(lint_cv_budget(&ordered, &input.budget));
    violations.extend(skill_violations(content, input.profile));
    Review { blocking: blocking_problems(content, input), violations }
}

/// Appelle le LLM une fois.
async fn call_once(
    input: &CvWritingInput<'_>,
    feedback: Option<WritingFeedback>,
    model: &str,
) -> Result<CvContent, AppError> {
    let message = with_feedback(user_message(input), feedback.as_ref());
    let raw = call_claude(&system_prompt(input), &message, Some(cv_content_schema(input)), model).await?;
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
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// `WritingOutcome<CvContent>`, entrées dans l'ordre d'affichage.
pub async fn write_cv(input: &CvWritingInput<'_>, model: &str) -> Result<WritingOutcome<CvContent>, AppError> {
    let prose = example_prose(input.example);
    let mut outcome = write_with_retries(
        "CV",
        |feedback| call_once(input, feedback, model),
        |content| review(content, input, &prose),
        |content| {
            content.map_prose(strip_dashes);
            retain_profile_skills(content, input.profile);
        },
    )
    .await?;
    outcome.content = order_by_selection(&outcome.content, input.selection);
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docgen::budgets::cv_budget;
    use crate::docgen::selection::select;
    use crate::models::{ProjetView, SkillView, SkillsGroup, UserView};

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
                adresse: None,
                code_postal: None,
                permis: None,
                description: None,
                linkedin: None,
                github: None,
                portfolio: None,
                interets: None,
                created_at: now,
                updated_at: now,
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

    fn matching() -> MatchingResult {
        MatchingResult {
            relevant_experiences: Vec::new(),
            relevant_projects: Vec::new(),
            relevant_skills: Vec::new(),
            summary_strategy: String::new(),
            gaps: Vec::new(),
        }
    }

    fn content(project_ids: &[i64], skills: &[&str]) -> CvContent {
        CvContent {
            title: "Titre".to_string(),
            summary: "Résumé".to_string(),
            experiences: Vec::new(),
            projects: project_ids.iter().map(|id| ItemBullets { id: *id, bullets: vec!["Fait".to_string()] }).collect(),
            skills_groups: vec![SkillsGroup {
                category: "Cat".to_string(),
                skills: skills.iter().map(|s| s.to_string()).collect(),
            }],
        }
    }

    #[test]
    fn identifiants_et_competences_valides() {
        let (profile, matching) = (profile(), matching());
        let selection = select(&profile, &matching, 2);
        let example = CvData::default();
        let input = CvWritingInput {
            offer_analysis: "{}",
            matching: &matching,
            selection: &selection,
            profile: &profile,
            example: &example,
            budget: cv_budget(&example, 1),
            target_lang: "fr",
            example_lang: "fr",
        };
        assert!(blocking_problems(&content(&[2, 1], &["rust", "SQL"]), &input).is_empty());
        assert!(skill_violations(&content(&[2, 1], &["rust", "SQL"]), &profile).is_empty());

        let problems = blocking_problems(&content(&[1, 1, 9], &["Kubernetes"]), &input);
        assert!(problems.iter().any(|p| p.contains("id 9 n'est pas")), "{problems:?}");
        assert!(problems.iter().any(|p| p.contains("id 2 manque")), "{problems:?}");
        assert!(problems.iter().any(|p| p.contains("id 1 apparaît 2 fois")), "{problems:?}");

        let skills = skill_violations(&content(&[1, 2], &["Kubernetes", "Rust", "rust"]), &profile);
        assert!(skills.iter().any(|v| v.message.contains("Kubernetes")), "{skills:?}");
        assert!(skills.iter().any(|v| v.message.contains("plusieurs groupes")), "{skills:?}");
    }

    #[test]
    fn competences_hors_profil_retirees_mecaniquement() {
        let mut cleaned = content(&[1], &["Kubernetes", "Rust", "rust", " "]);
        drop_empty_skills(&mut cleaned);
        retain_profile_skills(&mut cleaned, &profile());
        assert_eq!(cleaned.skills_groups[0].skills, vec!["Rust".to_string()]);

        let mut emptied = content(&[1], &["Kubernetes"]);
        retain_profile_skills(&mut emptied, &profile());
        assert!(emptied.skills_groups.is_empty(), "un groupe vidé disparaît");
    }

    #[test]
    fn ordre_d_affichage_de_la_selection() {
        let (profile, matching) = (profile(), matching());
        let selection = select(&profile, &matching, 2);
        let ordered = order_by_selection(&content(&[2, 1], &[]), &selection);
        let ids: Vec<i64> = ordered.projects.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }
}
