//! Formatage du profil candidat en texte condensé pour le prompt de scoring.
//! Ce n'est pas un dump JSON : le profil est répété dans chaque appel LLM
//! (un par batch de 5 offres), le texte est donc volontairement minimal.

// Importation des crates externes
use chrono::{Datelike, NaiveDate};

// Importation des modules internes
use findmyjob_core::models::{
    ExperienceView, FormationView, LangView, ProfileDto, SessionView, SkillView,
};

/// Formate le profil candidat en résumé texte (formation, expérience, skills,
/// langues). Les projets et coordonnées sont exclus : non pertinents pour
/// évaluer une offre.
///
/// # Arguments
///
/// * `profile` - Profil complet issu de `db::profile::build_profile`.
///
/// # Returns
///
/// Texte multi-lignes commençant par `Profil candidat :`. Les sections vides
/// sont omises.
pub fn format_profile_for_scoring(profile: &ProfileDto) -> String {
    let mut lines = vec!["Profil candidat :".to_string()];

    if !profile.formations.is_empty() {
        lines.push(format!("- Formation : {}", format_formations(&profile.formations)));
    }
    if !profile.experiences.is_empty() {
        lines.push(format!("- Expérience : {}", format_experiences(&profile.experiences)));
    }
    if !profile.skills.is_empty() {
        lines.push(format!("- Skills : {}", format_skills(&profile.skills)));
    }
    if !profile.langues.is_empty() {
        lines.push(format!("- Langues : {}", format_langues(&profile.langues)));
    }

    lines.join("\n")
}

/// Formate le profil candidat suivi de l'intention de recherche de la session.
///
/// # Arguments
///
/// * `profile` - Profil complet issu de `db::profile::build_profile`.
/// * `session` - Session dont `zone` et `contexte` décrivent la recherche.
///
/// # Returns
///
/// Profil condensé + bloc `Recherche en cours :` (omis si `zone` et
/// `contexte` sont tous deux vides).
pub fn format_scoring_context(profile: &ProfileDto, session: &SessionView) -> String {
    let mut text = format_profile_for_scoring(profile);

    let zone = non_empty(session.zone.as_deref());
    let contexte = non_empty(session.contexte.as_deref());
    if zone.is_none() && contexte.is_none() {
        return text;
    }

    text.push_str("\n\nRecherche en cours :");
    if let Some(zone) = zone {
        text.push_str(&format!("\n- Zone : {zone}"));
    }
    if let Some(contexte) = contexte {
        text.push_str(&format!("\n- Contexte : {contexte}"));
    }
    text
}

/// Retourne la valeur trimée, ou `None` si absente ou vide.
fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

/// Formations : `titre (école, début–fin)`, séparées par des virgules.
fn format_formations(formations: &[FormationView]) -> String {
    formations
        .iter()
        .map(|f| {
            let details = [f.ecole.clone(), format_year_range(f.date_debut, f.date_fin)]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
            if details.is_empty() {
                f.titre.clone()
            } else {
                format!("{} ({details})", f.titre)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Plage d'années `2022–2027` ; année unique si début et fin sont identiques.
fn format_year_range(debut: Option<NaiveDate>, fin: Option<NaiveDate>) -> Option<String> {
    match (debut, fin) {
        (Some(d), Some(f)) if d.year() == f.year() => Some(d.year().to_string()),
        (Some(d), Some(f)) => Some(format!("{}–{}", d.year(), f.year())),
        (Some(d), None) => Some(format!("{}–en cours", d.year())),
        (None, Some(f)) => Some(f.year().to_string()),
        (None, None) => None,
    }
}

/// Expériences : `poste chez entreprise (type, durée)`, séparées par des virgules.
fn format_experiences(experiences: &[ExperienceView]) -> String {
    experiences
        .iter()
        .map(|e| {
            let mut entry = e.poste.clone();
            if let Some(entreprise) = non_empty(e.entreprise.as_deref()) {
                entry.push_str(&format!(" chez {entreprise}"));
            }
            let details = [e.type_contrat.clone(), format_duration(e.date_debut, e.date_fin)]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
            if !details.is_empty() {
                entry.push_str(&format!(" ({details})"));
            }
            entry
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Durée approximative en mois ; `en cours` si aucune date de fin.
fn format_duration(debut: Option<NaiveDate>, fin: Option<NaiveDate>) -> Option<String> {
    let debut = debut?;
    let Some(fin) = fin else {
        return Some("en cours".to_string());
    };
    let months = (fin.year() - debut.year()) * 12 + fin.month() as i32 - debut.month() as i32 + 1;
    (months > 0).then(|| format!("{months} mois"))
}

/// Skills groupés par domaine (ordre d'apparition), groupes séparés par ` | `.
fn format_skills(skills: &[SkillView]) -> String {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for skill in skills {
        match groups.iter_mut().find(|(domaine, _)| *domaine == skill.domaine) {
            Some((_, noms)) => noms.push(&skill.nom),
            None => groups.push((&skill.domaine, vec![&skill.nom])),
        }
    }
    groups
        .iter()
        .map(|(_, noms)| noms.join(", "))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Langues : `nom (niveau)`, séparées par des virgules.
fn format_langues(langues: &[LangView]) -> String {
    langues
        .iter()
        .filter_map(|l| {
            let nom = non_empty(l.nom.as_deref())?;
            Some(match non_empty(l.level.as_deref()) {
                Some(level) => format!("{nom} ({level})"),
                None => nom.to_string(),
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}
