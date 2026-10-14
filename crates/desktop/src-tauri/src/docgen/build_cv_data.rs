//! Assemblage déterministe des données d'un CV (`CvData`) : fusion du
//! profil (via les identifiants de la sélection), du contenu rédigé, des
//! libellés et des périodes formatées. Une information que le profil n'a
//! pas (identité, liste vide) est reprise du document de référence. Aucun
//! appel LLM.

// Importation des modules internes
use crate::docgen::document_fallback::list_or_document;
use crate::docgen::identity::{build_identity, lines, text};
use findmyjob_core::docgen::period_format::{format_period, PeriodStyleText};
use findmyjob_core::docgen::selection::Selection;
use findmyjob_core::models::{
    CvContent, CvData, CvExperience, CvFormation, CvLabels, CvLangue, CvProject, ItemBullets, PeriodStyle, ProfileDto,
};

/// Ce que l'assemblage du CV reprend de la référence.
///
/// # Attributes
///
/// * `labels` - Libellés dans la langue du CV.
/// * `period_style` - Granularité des périodes par section.
/// * `separator` - Séparateur de période de la référence.
/// * `lang` - Langue du CV.
/// * `document` - Données d'exemple de la référence, reprises quand le
///   profil est vide sur une information.
pub struct CvLayout<'a> {
    pub document: &'a CvData,
    pub labels: &'a CvLabels,
    pub period_style: &'a PeriodStyle,
    pub separator: &'a str,
    pub lang: &'a str,
}

/// Bullets rédigés pour une entrée, vide si le contenu n'en a pas.
fn bullets_for(items: &[ItemBullets], id: i64) -> Vec<String> {
    items.iter().find(|item| item.id == id).map(|item| item.bullets.clone()).unwrap_or_default()
}

/// Assemble les données du CV.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat (source des faits qu'il connaît).
/// * `selection` - Entrées retenues, dans l'ordre d'affichage.
/// * `content` - Contenu rédigé (prose).
/// * `layout` - Libellés, style des périodes et langue.
///
/// # Returns
///
/// `CvData` prêt à être sérialisé en `data.json`.
pub fn build_cv_data(profile: &ProfileDto, selection: &Selection, content: &CvContent, layout: &CvLayout<'_>) -> CvData {
    let style = PeriodStyleText {
        months_short: layout.labels.months_short.clone(),
        present: layout.labels.present.clone(),
        separator: layout.separator.to_string(),
    };

    let experiences = selection
        .experiences
        .iter()
        .map(|s| CvExperience {
            poste: s.item.poste.trim().to_string(),
            entreprise: text(s.item.entreprise.as_ref()),
            lieu: text(s.item.lieu.as_ref()),
            period: format_period(s.item.date_debut, s.item.date_fin, layout.period_style.experiences, &style),
            bullets: bullets_for(&content.experiences, s.item.id),
        })
        .collect();

    let projects = selection
        .projects
        .iter()
        .map(|s| CvProject {
            titre: s.item.titre.trim().to_string(),
            pour_qui: text(s.item.pour_qui.as_ref()),
            period: format_period(s.item.date_debut, s.item.date_fin, layout.period_style.projects, &style),
            bullets: bullets_for(&content.projects, s.item.id),
        })
        .collect();

    let formations = profile
        .formations
        .iter()
        .map(|f| CvFormation {
            titre: f.titre.trim().to_string(),
            ecole: text(f.ecole.as_ref()),
            period: format_period(f.date_debut, f.date_fin, layout.period_style.formations, &style),
            details: lines(f.description.as_ref()),
        })
        .collect();

    let langues = profile
        .langues
        .iter()
        .filter(|l| !text(l.nom.as_ref()).is_empty())
        .map(|l| CvLangue { nom: text(l.nom.as_ref()), level: text(l.level.as_ref()) })
        .collect();

    let document = layout.document;
    CvData {
        identity: build_identity(&profile.user, &content.title, &document.identity),
        lang: layout.lang.to_string(),
        summary: content.summary.clone(),
        skills_groups: list_or_document(content.skills_groups.clone(), &document.skills_groups),
        experiences: list_or_document(experiences, &document.experiences),
        projects: list_or_document(projects, &document.projects),
        formations: list_or_document(formations, &document.formations),
        langues: list_or_document(langues, &document.langues),
        interets: list_or_document(lines(profile.user.interets.as_ref()), &document.interets),
        labels: layout.labels.clone(),
    }
}
