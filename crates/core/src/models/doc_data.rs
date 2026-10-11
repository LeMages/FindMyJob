//! Schémas de données canoniques des documents générés (CV, lettre de
//! motivation), identiques pour tous les utilisateurs.
//!
//! Un template Typst (`template.typ`, produit à l'upload d'une référence) lit
//! toutes ses données depuis un fichier `data.json` qui suit l'un de ces
//! schémas : `CvData` pour un CV, `LmData` pour une lettre. Un champ
//! optionnel absent vaut la chaîne vide (ou une liste vide), jamais `null` :
//! le template teste `!= ""` pour faire disparaître la ligne correspondante.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// ============================================================================
// Partie commune
// ============================================================================

/// Coordonnées du candidat, issues du profil (`profile.users`).
///
/// # Attributes
///
/// * `prenom`, `nom` - Identité, valeurs naturelles (le template applique
///   lui-même une éventuelle mise en majuscules).
/// * `email`, `telephone` - Coordonnées.
/// * `adresse` - Lieu du candidat, texte libre du profil (adresse complète,
///   ville seule…), affiché tel quel, jamais découpé : coordonnées et ligne
///   de date de la lettre.
/// * `linkedin`, `github`, `portfolio` - Liens, tels que saisis dans le profil.
/// * `permis` - Vrai si le candidat a le permis B (affiché via un libellé).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Identity {
    pub prenom: String,
    pub nom: String,
    pub email: String,
    pub telephone: String,
    pub adresse: String,
    pub linkedin: String,
    pub github: String,
    pub portfolio: String,
    pub permis: bool,
}

// ============================================================================
// CV
// ============================================================================

/// Groupe de compétences affiché sur une ligne du CV.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SkillsGroup {
    pub category: String,
    pub skills: Vec<String>,
}

/// Entrée de la section « Expérience » du CV.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvExperience {
    pub poste: String,
    pub entreprise: String,
    pub lieu: String,
    pub period: String,
    pub bullets: Vec<String>,
}

/// Entrée de la section « Projets » du CV.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvProject {
    pub titre: String,
    pub pour_qui: String,
    pub period: String,
    pub bullets: Vec<String>,
}

/// Entrée de la section « Formation » du CV.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvFormation {
    pub titre: String,
    pub ecole: String,
    pub period: String,
    pub details: Vec<String>,
}

/// Langue parlée, avec son niveau.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvLangue {
    pub nom: String,
    pub level: String,
}

/// Textes fixes d'un CV (titres de sections, libellés), dans la langue du
/// document. Chaîne vide = absent de la référence.
///
/// # Attributes
///
/// * `section_*` - Titres des sections, tels qu'écrits dans la référence.
/// * `interets_label` - Préfixe de la ligne des centres d'intérêt.
/// * `permis` - Texte affiché quand `identity.permis` est vrai (ex. « Permis B »).
/// * `present` - Borne de fin d'une période en cours (ex. « aujourd'hui »).
/// * `months_short` - Les 12 abréviations de mois utilisées dans les périodes.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvLabels {
    pub section_profile: String,
    pub section_skills: String,
    pub section_experiences: String,
    pub section_projects: String,
    pub section_formations: String,
    pub section_langues: String,
    pub interets_label: String,
    pub permis: String,
    pub present: String,
    pub months_short: Vec<String>,
}

/// Données complètes d'un CV, lues par son template.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvData {
    pub identity: Identity,
    pub lang: String,
    pub title: String,
    pub summary: String,
    pub skills_groups: Vec<SkillsGroup>,
    pub experiences: Vec<CvExperience>,
    pub projects: Vec<CvProject>,
    pub formations: Vec<CvFormation>,
    pub langues: Vec<CvLangue>,
    pub interets: Vec<String>,
    pub labels: CvLabels,
}

/// Granularité d'affichage d'une période (mois + année, ou année seule).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PeriodGranularity {
    #[default]
    Month,
    Year,
}

/// Granularité des périodes par section, détectée dans la référence.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PeriodStyle {
    pub experiences: PeriodGranularity,
    pub projects: PeriodGranularity,
    pub formations: PeriodGranularity,
}

/// Contenu d'un CV de référence rangé dans le schéma canonique
/// (`example_data.json`), avec les métadonnées détectées à l'upload.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvExampleData {
    #[serde(flatten)]
    pub data: CvData,
    pub detected_lang: String,
    pub period_style: PeriodStyle,
}

// ============================================================================
// Lettre de motivation
// ============================================================================

/// Destinataire de la lettre : entreprise et lieu de la candidature. La
/// lettre ne mentionne jamais de contact.
///
/// # Attributes
///
/// * `company` - Entreprise de la candidature.
/// * `location` - Lieu de la candidature.
/// * `contact_name`, `contact_role` - Toujours vides. Gardés uniquement pour
///   les templates convertis avant l'abandon du contact dans la lettre, qui
///   lisent encore ces clés (une clé absente ferait échouer leur
///   compilation Typst). Absents du schéma de templatisation : `default`
///   les tolère manquants à la lecture.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Recipient {
    pub company: String,
    pub location: String,
    #[serde(default)]
    pub contact_name: String,
    #[serde(default)]
    pub contact_role: String,
}

/// Textes fixes d'une lettre, dans la langue du document. Chaîne vide =
/// absent de la référence.
///
/// # Attributes
///
/// * `subject_prefix` - Préfixe de l'objet (ex. « Objet : »).
/// * `salutation` - Formule d'appel (ex. « Madame, Monsieur, »).
/// * `closing` - Formule de politesse finale, en entier.
/// * `date_prefix` - Mot entre le lieu (`identity.adresse`) et la date (ex. « le »).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LmLabels {
    pub subject_prefix: String,
    pub salutation: String,
    pub closing: String,
    pub date_prefix: String,
}

/// Données complètes d'une lettre de motivation, lues par son template.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LmData {
    pub identity: Identity,
    pub lang: String,
    pub recipient: Recipient,
    pub date: String,
    pub subject: String,
    pub paragraphs: Vec<String>,
    pub labels: LmLabels,
}

/// Contenu d'une lettre de référence rangé dans le schéma canonique
/// (`example_data.json`), avec la langue détectée à l'upload.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LmExampleData {
    #[serde(flatten)]
    pub data: LmData,
    pub detected_lang: String,
}
