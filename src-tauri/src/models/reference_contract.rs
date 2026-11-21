//! Contrat FindMyJob d'une référence (CV ou lettre de motivation) : la
//! métadonnée `#metadata((...)) <findmyjob>` déclarée dans le template, et
//! ce que l'application en garde à côté de la version (`contract.json`).
//!
//! Le template d'une référence conforme est autonome : il compile seul avec
//! ses valeurs d'exemple, et lit les vraies données dans `sys.inputs.data`
//! quand l'application les fournit. La métadonnée est lue après compilation
//! par l'introspecteur Typst, sans appel LLM.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::models::PeriodGranularity;

/// Version courante du contrat. Une référence déclarant une autre version
/// (ou importée avant le contrat) doit être réimportée.
pub const CONTRACT_VERSION: u32 = 1;

/// Valeur de `contract` attendue pour un CV.
pub const CONTRACT_CV: &str = "findmyjob/cv";

/// Valeur de `contract` attendue pour une lettre de motivation.
pub const CONTRACT_LM: &str = "findmyjob/lettre";

/// Nom de l'étiquette Typst de la métadonnée (`<findmyjob>`).
pub const METADATA_LABEL: &str = "findmyjob";

/// Nom du fichier du contrat extrait, dans le dossier de la version.
pub const CONTRACT_FILE: &str = "contract.json";

/// Valeur de `contract` attendue pour un type de document.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// `CONTRACT_CV` ou `CONTRACT_LM`.
pub fn expected_contract(doc_type: &str) -> &'static str {
    if doc_type == "cv" {
        CONTRACT_CV
    } else {
        CONTRACT_LM
    }
}

/// Rôle d'un emplacement rédigé.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlotMode {
    /// Rédigé à partir de l'offre.
    Variable,
    /// Réorganisé depuis le profil selon l'offre, sans rien inventer.
    Adaptable,
}

impl SlotMode {
    /// Mode lu dans la métadonnée (`"variable"` ou `"adaptable"`).
    ///
    /// # Returns
    ///
    /// `None` si le mode est inconnu.
    pub fn parse(mode: &str) -> Option<Self> {
        match mode {
            "variable" => Some(SlotMode::Variable),
            "adaptable" => Some(SlotMode::Adaptable),
            _ => None,
        }
    }
}

/// Emplacement rédigé déclaré par le template.
///
/// # Attributes
///
/// * `field` - Champ de prose : `identity.title`, `summary` (CV), `subject`,
///   `paragraphs[i]` (lettre).
/// * `mode` - Rôle de l'emplacement.
/// * `instruction` - Consigne propre à l'emplacement (chaîne vide si aucune).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Slot {
    pub field: String,
    pub mode: SlotMode,
    pub instruction: String,
}

/// Forme d'affichage du mois dans une période.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonthStyle {
    /// Abréviation : « janv. 2025 », « Jan 2025 ».
    #[default]
    Short,
    /// Nom complet : « janvier 2025 », « January 2025 ».
    Long,
    /// Numérique : « 01.2025 » (séparateur `numeric_separator`).
    Numeric,
}

/// Granularité des périodes par section du CV.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PeriodGranularityBySection {
    pub experiences: PeriodGranularity,
    pub projects: PeriodGranularity,
    pub formations: PeriodGranularity,
}

/// Format des périodes du CV, déclaré par le template et appliqué par Rust.
///
/// # Attributes
///
/// * `granularity` - Mois + année, ou année seule, par section.
/// * `separator` - Séparateur entre début et fin (ex. `" - "`).
/// * `month_style` - Forme du mois (abrégé, complet, numérique).
/// * `numeric_separator` - Entre le mois et l'année en forme numérique (ex. `"."`).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PeriodSpec {
    pub granularity: PeriodGranularityBySection,
    pub separator: String,
    #[serde(default)]
    pub month_style: MonthStyle,
    #[serde(default = "default_numeric_separator")]
    pub numeric_separator: String,
}

/// Séparateur numérique par défaut (« 01/2025 »).
fn default_numeric_separator() -> String {
    "/".to_string()
}

/// Emplacement tel qu'écrit dans la métadonnée, avant validation du mode.
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawSlot {
    pub field: String,
    pub mode: String,
    #[serde(default)]
    pub instruction: String,
}

/// Métadonnée `<findmyjob>` telle que déclarée par le template.
///
/// # Attributes
///
/// * `contract` - `"findmyjob/cv"` ou `"findmyjob/lettre"`.
/// * `version` - Version du contrat (`CONTRACT_VERSION`).
/// * `languages` - Langues rendues par le template.
/// * `example` - Valeurs d'exemple (même objet que `#let example`), validées à part.
/// * `guidance` - Consignes générales de rédaction.
/// * `slots` - Emplacements rédigés.
/// * `period` - Format des périodes (CV seulement).
#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawMetadata {
    pub contract: String,
    pub version: u32,
    pub languages: Vec<String>,
    pub example: serde_json::Value,
    #[serde(default)]
    pub guidance: String,
    #[serde(default)]
    pub slots: Vec<RawSlot>,
    #[serde(default)]
    pub period: Option<PeriodSpec>,
}

/// Contrat extrait d'une référence validée, enregistré dans `contract.json`
/// à côté de la version : la génération n'a pas besoin de recompiler la
/// référence pour connaître son exemple.
///
/// # Attributes
///
/// * `contract_version` - Version du contrat de la référence.
/// * `languages` - Langues rendues par le template.
/// * `example` - Valeurs d'exemple, au schéma des données du document.
/// * `guidance` - Consignes générales de rédaction.
/// * `slots` - Emplacements rédigés.
/// * `period` - Format des périodes (CV seulement).
/// * `page_count` - Nombre de pages du rendu de l'exemple (plafond des documents générés).
/// * `example_text` - Texte rendu de l'exemple, par langue (modèle de style du rédacteur).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredContract<E> {
    pub contract_version: u32,
    pub languages: Vec<String>,
    pub example: E,
    pub guidance: String,
    pub slots: Vec<Slot>,
    pub period: Option<PeriodSpec>,
    pub page_count: usize,
    pub example_text: BTreeMap<String, String>,
}
