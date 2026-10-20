//! Mise en forme des faits du profil retenus par la sélection, pour les
//! prompts de rédaction (CV et lettre). Les descriptions sont données dans
//! la langue du document (`CvEntryTexts`, traduits fidèlement si besoin) :
//! elles servent de matière au résumé et à la lettre, le CV les affiche
//! telles quelles. Les noms officiels (poste, entreprise, école, titre de
//! projet) restent ceux du profil.

// Importation des modules internes
use crate::docgen::entry_texts::CvEntryTexts;
use crate::docgen::selection::Selection;
use crate::models::{ExperienceView, MatchingResult, ProfileDto, ProjetView};

/// Valeur d'un champ optionnel, ou « non renseigné ».
fn or_unknown(value: Option<&str>) -> &str {
    value.map(str::trim).filter(|v| !v.is_empty()).unwrap_or("non renseigné")
}

/// Dates d'une entrée au format ISO, pour situer les faits dans le temps.
fn dates(start: Option<chrono::NaiveDate>, end: Option<chrono::NaiveDate>) -> String {
    let fmt = |d: Option<chrono::NaiveDate>, missing: &str| d.map(|d| d.to_string()).unwrap_or_else(|| missing.to_string());
    format!("du {} au {}", fmt(start, "?"), fmt(end, "aujourd'hui"))
}

/// Lignes d'une description, jointes, ou « non renseigné ».
fn joined(lines: &[String]) -> String {
    if lines.is_empty() {
        "non renseigné".to_string()
    } else {
        lines.join(" ")
    }
}

/// Fiche d'une expérience retenue.
fn experience_facts(e: &ExperienceView, texts: &CvEntryTexts) -> String {
    format!(
        "Expérience id {} : {} chez {} ({}), {}, contrat {}\n  Description (source des faits) : {}\n",
        e.id,
        e.poste,
        or_unknown(e.entreprise.as_deref()),
        or_unknown(e.lieu.as_deref()),
        dates(e.date_debut, e.date_fin),
        or_unknown(e.type_contrat.as_deref()),
        joined(&texts.experience_lines(e.id)),
    )
}

/// Fiche d'un projet retenu.
fn project_facts(p: &ProjetView, texts: &CvEntryTexts) -> String {
    let (pour_qui, lines) = match texts.project(p.id) {
        Some(t) => (t.pour_qui.clone(), joined(&t.lines)),
        None => (or_unknown(p.pour_qui.as_deref()).to_string(), "non renseigné".to_string()),
    };
    format!(
        "Projet id {} : {} (pour {}), {}\n  Description (source des faits) : {}\n  Technologies : {}\n",
        p.id,
        p.titre,
        or_unknown(Some(&pour_qui)),
        dates(p.date_debut, p.date_fin),
        lines,
        or_unknown(p.stack.as_deref()),
    )
}

/// Bloc des faits du profil : présentation, formations, expériences et
/// projets retenus, compétences.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat.
/// * `selection` - Entrées retenues.
/// * `matching` - Résultat du matching (compétences qui répondent à l'offre, manques).
/// * `texts` - Descriptions des entrées, dans la langue du document.
///
/// # Returns
///
/// Bloc de texte à insérer dans le prompt.
pub fn profile_facts_block(
    profile: &ProfileDto,
    selection: &Selection,
    matching: &MatchingResult,
    texts: &CvEntryTexts,
) -> String {
    let formations = profile
        .formations
        .iter()
        .map(|f| {
            format!(
                "- {} ({}), {}. Détails : {}",
                f.titre,
                or_unknown(f.ecole.as_deref()),
                dates(f.date_debut, f.date_fin),
                joined(&texts.formation_lines(f.id))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let experiences = selection.experiences.iter().map(|e| experience_facts(e, texts)).collect::<Vec<_>>().join("\n");
    let projects = selection.projects.iter().map(|p| project_facts(p, texts)).collect::<Vec<_>>().join("\n");
    let skills = profile
        .skills
        .iter()
        .map(|s| format!("{} ({})", s.nom, s.domaine))
        .collect::<Vec<_>>()
        .join(", ");
    let matched_skills = matching
        .relevant_skills
        .iter()
        .map(|s| format!("{} ({})", s.skill, s.match_type))
        .collect::<Vec<_>>()
        .join(", ");
    let gaps = if matching.gaps.is_empty() { "aucune".to_string() } else { matching.gaps.join(", ") };

    format!(
        "PROFIL DU CANDIDAT (seule source des faits)\n\
         Présentation : {}\n\
         Formations :\n{formations}\n\n\
         Expériences retenues, dans l'ordre d'affichage :\n{experiences}\n\
         Projets retenus, dans l'ordre d'affichage :\n{projects}\n\
         Compétences du profil (nom, domaine) : {skills}\n\
         Compétences du profil qui répondent à l'offre : {matched_skills}\n\
         Compétences demandées par l'offre et absentes du profil, à ne jamais affirmer : {gaps}\n\
         Angle conseillé pour le résumé ou l'accroche : {}",
        or_unknown(profile.user.description.as_deref()),
        matching.summary_strategy
    )
}
