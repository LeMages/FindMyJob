//! Schémas de données canoniques des documents générés (CV, lettre de
//! motivation), identiques pour tous les utilisateurs.
//!
//! Une référence au contrat FindMyJob lit dans `sys.inputs.data` (JSON) des
//! données qui suivent l'un de ces schémas (`CvData` pour un CV, `LmData`
//! pour une lettre) : tout ce que le profil, la session, l'offre ou la
//! rédaction fournissent. Ses libellés et textes fixes sont écrits dans le
//! template, dans chaque langue. Un champ optionnel absent vaut la chaîne
//! vide (ou une liste vide), jamais `null` : le template teste `!= ""` pour
//! faire disparaître la ligne correspondante. Le schéma est aussi celui des
//! valeurs d'exemple du template (`documents::reference_contract::example_schema`).

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
/// * `title` - Sous-titre professionnel affiché sous le nom, commun au CV et
///   à la lettre. Pas tiré du profil : rédigé pour l'offre à la génération.
/// * `email`, `telephone` - Coordonnées.
/// * `ville`, `zip_code` - Lieu du candidat (chaîne vide = non renseigné).
///   Le template compose lui-même le lieu affiché, dans l'ordre et avec la
///   ponctuation de la référence ; la ligne de date de la lettre n'utilise
///   que `ville`.
/// * `linkedin`, `github`, `portfolio` - Liens, tels que saisis dans le profil.
/// * `permis` - Vrai si le candidat a le permis B (affiché via un libellé du template).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Identity {
    pub prenom: String,
    pub nom: String,
    pub title: String,
    pub email: String,
    pub telephone: String,
    pub ville: String,
    pub zip_code: String,
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
///
/// # Attributes
///
/// * `poste`, `entreprise`, `lieu`, `period` - Intitulé, organisation, lieu et période.
/// * `intro` - Texte de la description avant le premier élément, rendu
///   sans symbole (lignes séparées par `\n`, chaîne vide si absent).
/// * `bullets` - Éléments de la description, chacun précédé de
///   `CvData.list_marker` ; un `\n` dans un élément est un retour à la
///   ligne dans le même élément.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvExperience {
    pub poste: String,
    pub entreprise: String,
    pub lieu: String,
    pub period: String,
    pub intro: String,
    pub bullets: Vec<String>,
}

/// Entrée de la section « Projets » du CV.
///
/// # Attributes
///
/// * `titre`, `pour_qui`, `period` - Intitulé, organisation et période.
/// * `stack` - Technologies du projet, telles que saisies dans le profil.
/// * `url` - Lien du projet, sous sa forme affichable.
/// * `intro` - Texte avant le premier élément, sans symbole (voir `CvExperience`).
/// * `bullets` - Éléments de la description du profil (voir `CvExperience`).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvProject {
    pub titre: String,
    pub pour_qui: String,
    pub period: String,
    pub stack: String,
    pub url: String,
    pub intro: String,
    pub bullets: Vec<String>,
}

/// Entrée de la section « Formation » du CV.
///
/// # Attributes
///
/// * `titre`, `ecole`, `period` - Diplôme, établissement et période.
/// * `intro` - Texte avant le premier élément, sans symbole (voir `CvExperience`).
/// * `details` - Éléments de la description (voir `CvExperience.bullets`).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvFormation {
    pub titre: String,
    pub ecole: String,
    pub period: String,
    pub intro: String,
    pub details: Vec<String>,
}

/// Langue parlée, avec son niveau.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvLangue {
    pub nom: String,
    pub level: String,
}

/// Données complètes d'un CV, lues par son template. `list_marker` est le
/// symbole placé devant chaque élément de `bullets` / `details` : à la
/// génération, celui du format de la session (`•` ou `-`) ; dans les
/// valeurs d'exemple, celui de la référence.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvData {
    pub identity: Identity,
    pub lang: String,
    pub summary: String,
    pub skills_groups: Vec<SkillsGroup>,
    pub experiences: Vec<CvExperience>,
    pub projects: Vec<CvProject>,
    pub formations: Vec<CvFormation>,
    pub langues: Vec<CvLangue>,
    pub list_marker: String,
}

/// Granularité d'affichage d'une période (mois + année, ou année seule).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PeriodGranularity {
    #[default]
    Month,
    Year,
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
/// * `extra_lines` - Autres lignes du destinataire (parties de la
///   localisation de la candidature avant la ville, voir
///   `docgen::recipient`) ; dans les valeurs d'exemple, celles de la référence.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Recipient {
    pub company: String,
    pub location: String,
    pub extra_lines: Vec<String>,
}

/// Données complètes d'une lettre de motivation, lues par son template.
/// `paragraphs` ne contient que les paragraphes rédigés, dans l'ordre : le
/// template place librement ses phrases fixes autour d'eux.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LmData {
    pub identity: Identity,
    pub lang: String,
    pub recipient: Recipient,
    pub date: String,
    pub subject: String,
    pub paragraphs: Vec<String>,
}
