//! Textes du profil tels qu'affichés par le CV : lignes de la description
//! des expériences, projets et formations (liste plate : intro, puis
//! éléments et leurs suites, avec la forme qui permet de reconstruire la
//! structure, voir `docgen::description_list`), organisation d'un projet
//! (`pour_qui`), nom et niveau des langues. Ce sont les
//! seuls textes du profil qu'une traduction peut modifier ; intitulés,
//! entreprises, écoles, lieux, stack et liens (noms officiels) sont
//! toujours lus tels quels dans le profil.

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::docgen::description_list::{parse_description, DescriptionList, ListShape};
use crate::docgen::identity::text;
use crate::docgen::selection::Selection;
use crate::models::ProfileDto;

/// Lignes de description d'une expérience ou d'une formation.
///
/// # Attributes
///
/// * `id` - Identifiant de l'entrée dans le profil.
/// * `lines` - Lignes de la description, à plat et sans symbole (intro,
///   puis chaque élément avec ses suites) : la seule forme envoyée à la
///   traduction, dont le contrat (une ligne traduite par ligne) ne change pas.
/// * `shape` - Répartition des lignes entre intro et éléments. Jamais
///   sérialisée : elle n'est pas confiée au LLM, et une traduction la
///   reprend de l'original (`CvEntryTexts::with_shapes_of`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryLines {
    pub id: i64,
    pub lines: Vec<String>,
    #[serde(skip)]
    pub shape: ListShape,
}

impl EntryLines {
    /// Lignes d'une entrée tirées de sa description du profil.
    fn from_description(id: i64, description: Option<&String>) -> Self {
        let list = parse_description(description.map(String::as_str));
        EntryLines { id, lines: list.flat_lines(), shape: list.shape() }
    }

    /// Description structurée (intro et éléments).
    ///
    /// # Returns
    ///
    /// La structure, vide si les lignes ne suivent pas la forme (cas qui
    /// n'arrive pas : la traduction est validée ligne à ligne).
    pub fn list(&self) -> DescriptionList {
        self.shape.apply(&self.lines).unwrap_or_default()
    }
}

/// Textes d'un projet.
///
/// # Attributes
///
/// * `id` - Identifiant du projet dans le profil.
/// * `pour_qui` - Organisation du projet (chaîne vide si absente).
/// * `lines` - Lignes de la description, à plat (voir `EntryLines`).
/// * `shape` - Répartition des lignes, jamais sérialisée (voir `EntryLines`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectTexts {
    pub id: i64,
    pub pour_qui: String,
    pub lines: Vec<String>,
    #[serde(skip)]
    pub shape: ListShape,
}

impl ProjectTexts {
    /// Description structurée (intro et éléments).
    ///
    /// # Returns
    ///
    /// La structure, vide si les lignes ne suivent pas la forme.
    pub fn list(&self) -> DescriptionList {
        self.shape.apply(&self.lines).unwrap_or_default()
    }
}

/// Textes d'une langue parlée.
///
/// # Attributes
///
/// * `id` - Identifiant de la langue dans le profil.
/// * `nom` - Nom de la langue (« Anglais »).
/// * `level` - Niveau (chaîne vide si absent).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageTexts {
    pub id: i64,
    pub nom: String,
    pub level: String,
}

/// Textes du profil affichés par le CV.
///
/// # Attributes
///
/// * `experiences` - Toutes les expériences retenues.
/// * `projects` - Les projets retenus.
/// * `formations` - Toutes les formations du profil.
/// * `langues` - Les langues parlées du profil (nom non vide).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CvEntryTexts {
    pub experiences: Vec<EntryLines>,
    pub projects: Vec<ProjectTexts>,
    pub formations: Vec<EntryLines>,
    pub langues: Vec<LanguageTexts>,
}

impl CvEntryTexts {
    /// Lignes à plat d'une expérience, vide si elle est absente.
    pub fn experience_lines(&self, id: i64) -> Vec<String> {
        self.experiences.iter().find(|e| e.id == id).map(|e| e.lines.clone()).unwrap_or_default()
    }

    /// Description structurée d'une expérience, vide si elle est absente.
    pub fn experience_list(&self, id: i64) -> DescriptionList {
        self.experiences.iter().find(|e| e.id == id).map(EntryLines::list).unwrap_or_default()
    }

    /// Lignes à plat d'une formation, vide si elle est absente.
    pub fn formation_lines(&self, id: i64) -> Vec<String> {
        self.formations.iter().find(|f| f.id == id).map(|f| f.lines.clone()).unwrap_or_default()
    }

    /// Description structurée d'une formation, vide si elle est absente.
    pub fn formation_list(&self, id: i64) -> DescriptionList {
        self.formations.iter().find(|f| f.id == id).map(EntryLines::list).unwrap_or_default()
    }

    /// Reprend la forme des descriptions de l'original, par identifiant :
    /// une traduction ne transporte que les lignes à plat.
    ///
    /// # Arguments
    ///
    /// * `original` - Textes dont la traduction est issue.
    ///
    /// # Returns
    ///
    /// Les mêmes textes, avec la forme de l'original pour chaque entrée.
    pub fn with_shapes_of(mut self, original: &CvEntryTexts) -> CvEntryTexts {
        let shape_of = |entries: &[EntryLines], id: i64| {
            entries.iter().find(|e| e.id == id).map(|e| e.shape.clone()).unwrap_or_default()
        };
        for entry in self.experiences.iter_mut() {
            entry.shape = shape_of(&original.experiences, entry.id);
        }
        for entry in self.formations.iter_mut() {
            entry.shape = shape_of(&original.formations, entry.id);
        }
        for project in self.projects.iter_mut() {
            project.shape = original.project(project.id).map(|p| p.shape.clone()).unwrap_or_default();
        }
        self
    }

    /// Textes d'un projet, `None` s'il est absent.
    pub fn project(&self, id: i64) -> Option<&ProjectTexts> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// Indique si aucun texte n'est présent : rien à traduire.
    pub fn is_blank(&self) -> bool {
        self.experiences.iter().chain(&self.formations).all(|e| e.lines.is_empty())
            && self.projects.iter().all(|p| p.lines.is_empty() && p.pour_qui.is_empty())
            && self.langues.is_empty()
    }
}

/// Textes des entrées dans la langue du profil.
///
/// # Arguments
///
/// * `profile` - Profil complet du candidat (formations).
/// * `selection` - Expériences et projets retenus.
///
/// # Returns
///
/// `CvEntryTexts` : chaque description découpée en intro et éléments
/// (`docgen::description_list`), mise à plat sans symbole.
pub fn source_texts(profile: &ProfileDto, selection: &Selection) -> CvEntryTexts {
    CvEntryTexts {
        experiences: selection
            .experiences
            .iter()
            .map(|e| EntryLines::from_description(e.id, e.description.as_ref()))
            .collect(),
        projects: selection
            .projects
            .iter()
            .map(|p| {
                let list = parse_description(p.description.as_deref());
                ProjectTexts {
                    id: p.id,
                    pour_qui: text(p.pour_qui.as_ref()),
                    lines: list.flat_lines(),
                    shape: list.shape(),
                }
            })
            .collect(),
        formations: profile
            .formations
            .iter()
            .map(|f| EntryLines::from_description(f.id, f.description.as_ref()))
            .collect(),
        langues: profile
            .langues
            .iter()
            .filter(|l| !text(l.nom.as_ref()).is_empty())
            .map(|l| LanguageTexts { id: l.id, nom: text(l.nom.as_ref()), level: text(l.level.as_ref()) })
            .collect(),
    }
}
