//! Assemblage déterministe des données d'un CV (`CvData`) : fusion du
//! profil (via les identifiants de la sélection), des textes des entrées
//! (descriptions, éventuellement traduites), du contenu rédigé, des
//! libellés et des périodes formatées. Les expériences, projets et
//! formations viennent uniquement du profil : une section vide dans le
//! profil reste vide. Aucun appel LLM.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des modules internes
use crate::docgen::entry_texts::CvEntryTexts;
use crate::docgen::identity::{build_identity, display_url, text};
use crate::docgen::period_format::{format_period, PeriodStyleText};
use crate::docgen::selection::Selection;
use crate::models::{
    CvContent, CvData, CvExperience, CvFormation, CvLabels, CvLangue, CvProject, PeriodStyle, ProfileDto,
};

/// Ce que l'assemblage du CV reprend de la référence.
///
/// # Attributes
///
/// * `labels` - Libellés dans la langue du CV.
/// * `period_style` - Granularité des périodes par section.
/// * `separator` - Séparateur de période de la référence.
/// * `lang` - Langue du CV.
/// * `texts` - Textes en dur de la référence, dans la langue du CV.
/// * `document` - Données d'exemple de la référence, reprises seulement
///   quand le profil est vide sur une information d'identité. Compétences,
///   langues, centres d'intérêt, expériences, projets et formations ne
///   viennent que du profil : vide dans le profil, la section est vide.
pub struct CvLayout<'a> {
    pub document: &'a CvData,
    pub labels: &'a CvLabels,
    pub period_style: &'a PeriodStyle,
    pub separator: &'a str,
    pub lang: &'a str,
    pub texts: &'a BTreeMap<String, String>,
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
/// * `layout` - Libellés, style des périodes et langue.
///
/// # Returns
///
/// `CvData` prêt à être sérialisé en `data.json`.
pub fn build_cv_data(
    profile: &ProfileDto,
    selection: &Selection,
    texts: &CvEntryTexts,
    content: &CvContent,
    layout: &CvLayout<'_>,
) -> CvData {
    let style = PeriodStyleText {
        months_short: layout.labels.months_short.clone(),
        present: layout.labels.present.clone(),
        separator: layout.separator.to_string(),
    };

    let experiences = selection
        .experiences
        .iter()
        .map(|e| CvExperience {
            poste: e.poste.trim().to_string(),
            entreprise: text(e.entreprise.as_ref()),
            lieu: text(e.lieu.as_ref()),
            period: format_period(e.date_debut, e.date_fin, layout.period_style.experiences, &style),
            bullets: texts.experience_lines(e.id),
        })
        .collect();

    let projects = selection
        .projects
        .iter()
        .map(|p| {
            // Sans texte pour ce projet (cas qui ne devrait pas arriver), le
            // profil reste la source : jamais la référence.
            let (pour_qui, bullets) = match texts.project(p.id) {
                Some(t) => (t.pour_qui.clone(), t.lines.clone()),
                None => (text(p.pour_qui.as_ref()), Vec::new()),
            };
            CvProject {
                titre: p.titre.trim().to_string(),
                pour_qui,
                period: format_period(p.date_debut, p.date_fin, layout.period_style.projects, &style),
                stack: text(p.stack.as_ref()),
                url: display_url(p.url.as_ref()),
                bullets,
            }
        })
        .collect();

    let formations = profile
        .formations
        .iter()
        .map(|f| CvFormation {
            titre: f.titre.trim().to_string(),
            ecole: text(f.ecole.as_ref()),
            period: format_period(f.date_debut, f.date_fin, layout.period_style.formations, &style),
            details: texts.formation_lines(f.id),
        })
        .collect();

    let langues = texts.langues.iter().map(|l| CvLangue { nom: l.nom.clone(), level: l.level.clone() }).collect();

    let document = layout.document;
    CvData {
        identity: build_identity(&profile.user, &content.title, &document.identity),
        lang: layout.lang.to_string(),
        summary: content.summary.clone(),
        skills_groups: content.skills_groups.clone(),
        experiences,
        projects,
        formations,
        langues,
        interets: texts.interets.clone(),
        labels: layout.labels.clone(),
        texts: layout.texts.clone(),
    }
}
