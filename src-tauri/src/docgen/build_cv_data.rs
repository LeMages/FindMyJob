//! Assemblage déterministe des données d'un CV (`CvData`) : fusion du
//! profil (via les identifiants de la sélection), des textes des entrées
//! (descriptions, éventuellement traduites), du contenu rédigé, des
//! et des périodes formatées. Les expériences, projets et
//! formations viennent uniquement du profil : une section vide dans le
//! profil reste vide. Aucun appel LLM.

// Importation des modules internes
use crate::docgen::entry_texts::CvEntryTexts;
use crate::docgen::identity::{build_identity, display_url, text, SenderLocation};
use crate::docgen::period_format::{format_period, PeriodFormat};
use crate::docgen::selection::Selection;
use crate::models::{
    CvContent, CvData, CvExperience, CvFormation, CvLangue, CvProject, PeriodGranularityBySection, ProfileDto,
};

/// Ce que l'assemblage du CV reprend de la référence.
///
/// # Attributes
///
/// * `granularity` - Granularité des périodes par section (déclarée par la référence).
/// * `period_format` - Forme du mois, séparateur et langue des périodes.
/// * `lang` - Langue du CV.
/// * `location` - Localisation de l'expéditeur (session), sans repli sur la référence.
/// * `list_marker` - Symbole des éléments de liste, celui du format de la
///   session (`docgen::description_list::list_marker`), jamais celui de la référence.
///
/// Identité, compétences, langues, expériences, projets et formations ne
/// viennent que du profil : vide dans le profil, la ligne ou la section est vide.
pub struct CvLayout<'a> {
    pub location: &'a SenderLocation,
    pub granularity: &'a PeriodGranularityBySection,
    pub period_format: &'a PeriodFormat,
    pub lang: &'a str,
    pub list_marker: &'a str,
}

/// Assemble les données du CV.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat (source des faits qu'il connaît).
/// * `selection` - Entrées retenues, dans l'ordre d'affichage.
/// * `texts` - Lignes de description et organisation des projets, dans la
///   langue du CV (`docgen::entry_texts`).
/// * `content` - Contenu rédigé (prose).
/// * `layout` - Format des périodes, langue, lieu et symbole de liste.
///
/// # Returns
///
/// `CvData` prêt à être sérialisé et injecté dans `sys.inputs.data`.
pub fn build_cv_data(
    profile: &ProfileDto,
    selection: &Selection,
    texts: &CvEntryTexts,
    content: &CvContent,
    layout: &CvLayout<'_>,
) -> CvData {
    let style = layout.period_format;
    let experiences = selection
        .experiences
        .iter()
        .map(|e| {
            let list = texts.experience_list(e.id);
            CvExperience {
                poste: e.poste.trim().to_string(),
                entreprise: text(e.entreprise.as_ref()),
                lieu: text(e.lieu.as_ref()),
                period: format_period(e.date_debut, e.date_fin, layout.granularity.experiences, style),
                intro: list.intro_text(),
                bullets: list.item_texts(),
            }
        })
        .collect();

    let projects = selection
        .projects
        .iter()
        .map(|p| {
            // Sans texte pour ce projet (cas qui ne devrait pas arriver), le
            // profil reste la source : jamais la référence.
            let (pour_qui, list) = match texts.project(p.id) {
                Some(t) => (t.pour_qui.clone(), t.list()),
                None => (text(p.pour_qui.as_ref()), Default::default()),
            };
            CvProject {
                titre: p.titre.trim().to_string(),
                pour_qui,
                period: format_period(p.date_debut, p.date_fin, layout.granularity.projects, style),
                stack: text(p.stack.as_ref()),
                url: display_url(p.url.as_ref()),
                intro: list.intro_text(),
                bullets: list.item_texts(),
            }
        })
        .collect();

    let formations = profile
        .formations
        .iter()
        .map(|f| {
            let list = texts.formation_list(f.id);
            CvFormation {
                titre: f.titre.trim().to_string(),
                ecole: text(f.ecole.as_ref()),
                period: format_period(f.date_debut, f.date_fin, layout.granularity.formations, style),
                intro: list.intro_text(),
                details: list.item_texts(),
            }
        })
        .collect();

    let langues = texts.langues.iter().map(|l| CvLangue { nom: l.nom.clone(), level: l.level.clone() }).collect();

    CvData {
        identity: build_identity(&profile.user, layout.location, &content.title),
        lang: layout.lang.to_string(),
        summary: content.summary.clone(),
        skills_groups: content.skills_groups.clone(),
        experiences,
        projects,
        formations,
        langues,
        list_marker: layout.list_marker.to_string(),
    }
}
