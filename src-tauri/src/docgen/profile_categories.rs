//! Catégories d'information que le profil fournit au CV : source unique de
//! leur nom, de leur description pour le prompt de l'étape A, de leurs
//! chemins dans le schéma canonique et de la règle « affichée dans
//! l'exemple de la référence mais vide dans le profil » (unité retirée et
//! avertissement, jamais de repli sur la référence).
//!
//! Toute ligne d'une référence qui ne relève d'aucune de ces catégories est
//! un texte en dur (`data.texts`), traduit mais jamais tiré du profil.

// Importation des modules internes
use crate::models::{CvData, ProfileDto};

/// Catégorie d'information du profil affichée par le CV : une variante par
/// catégorie, dans l'ordre des champs du schéma canonique (`all()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileCategory {
    Identite,
    Experiences,
    Projets,
    Formations,
    Competences,
    Langues,
    Permis,
}

impl ProfileCategory {
    /// Toutes les catégories, dans l'ordre du schéma canonique.
    pub fn all() -> [ProfileCategory; 7] {
        [
            ProfileCategory::Identite,
            ProfileCategory::Experiences,
            ProfileCategory::Projets,
            ProfileCategory::Formations,
            ProfileCategory::Competences,
            ProfileCategory::Langues,
            ProfileCategory::Permis,
        ]
    }

    /// Nom français de la catégorie.
    pub fn french_name(self) -> &'static str {
        match self {
            ProfileCategory::Identite => "identité",
            ProfileCategory::Experiences => "expériences",
            ProfileCategory::Projets => "projets",
            ProfileCategory::Formations => "formations",
            ProfileCategory::Competences => "compétences",
            ProfileCategory::Langues => "langues",
            ProfileCategory::Permis => "permis de conduire",
        }
    }

    /// Chemins de la catégorie dans le schéma canonique des données du CV.
    pub fn schema_paths(self) -> &'static [&'static str] {
        match self {
            ProfileCategory::Identite => &["identity"],
            ProfileCategory::Experiences => &["experiences"],
            ProfileCategory::Projets => &["projects"],
            ProfileCategory::Formations => &["formations"],
            ProfileCategory::Competences => &["skills_groups"],
            ProfileCategory::Langues => &["langues"],
            ProfileCategory::Permis => &["identity.permis"],
        }
    }

    /// Description de la catégorie pour le prompt de l'étape A.
    pub fn prompt_description(self) -> &'static str {
        match self {
            ProfileCategory::Identite => {
                "coordonnées du candidat affichées dans l'original. identity.title : sous-titre professionnel affiché sous le nom, chaîne vide si l'original n'en a pas."
            }
            ProfileCategory::Experiences => {
                "une entrée par expérience professionnelle, avec ses bullets (`bullets`)."
            }
            ProfileCategory::Projets => {
                "une entrée par projet, avec ses bullets. `pour_qui` est l'organisation affichée pour un projet. projects.stack et projects.url : technologies et lien affichés."
            }
            ProfileCategory::Formations => {
                "une entrée par formation ; `details` reçoit ses lignes de détail (spécialité, cours, échange)."
            }
            ProfileCategory::Competences => "une entrée par ligne de compétences, la catégorie puis les compétences.",
            ProfileCategory::Langues => "nom et niveau de chaque langue parlée.",
            ProfileCategory::Permis => {
                "vrai seulement si l'original affiche le permis de conduire ; le texte affiché est `labels.permis`."
            }
        }
    }

    /// Indique si le profil n'a aucune information pour cette catégorie.
    ///
    /// L'identité n'est jamais « vide » ici : son repli sur la référence
    /// relève d'une règle à part (`docgen::contact_fields`, sans repli).
    ///
    /// # Arguments
    ///
    /// * `profile` - Profil complet du candidat.
    ///
    /// # Returns
    ///
    /// Vrai si le profil est vide pour la catégorie. Le permis n'est vide
    /// que lorsqu'il n'est pas renseigné (NULL) : un « non » est une vraie
    /// réponse.
    pub fn is_empty_in_profile(self, profile: &ProfileDto) -> bool {
        match self {
            ProfileCategory::Identite => false,
            ProfileCategory::Experiences => profile.experiences.is_empty(),
            ProfileCategory::Projets => profile.projets.is_empty(),
            ProfileCategory::Formations => profile.formations.is_empty(),
            ProfileCategory::Competences => profile.skills.is_empty(),
            ProfileCategory::Langues => profile.langues.iter().all(|l| l.nom.as_deref().unwrap_or("").trim().is_empty()),
            ProfileCategory::Permis => profile.user.permis.is_none(),
        }
    }

    /// Indique si l'exemple de la référence affiche cette catégorie.
    ///
    /// # Arguments
    ///
    /// * `example` - Données d'exemple du CV de référence.
    pub fn shown_in_example(self, example: &CvData) -> bool {
        match self {
            ProfileCategory::Identite => true,
            ProfileCategory::Experiences => !example.experiences.is_empty(),
            ProfileCategory::Projets => !example.projects.is_empty(),
            ProfileCategory::Formations => !example.formations.is_empty(),
            ProfileCategory::Competences => !example.skills_groups.is_empty(),
            ProfileCategory::Langues => !example.langues.is_empty(),
            ProfileCategory::Permis => example.identity.permis,
        }
    }

    /// Avertissement affiché quand la catégorie est retirée du CV.
    pub fn warning(self) -> String {
        let reason = match self {
            ProfileCategory::Identite => return String::new(),
            ProfileCategory::Experiences => "Votre profil ne contient aucune expérience",
            ProfileCategory::Projets => "Votre profil ne contient aucun projet",
            ProfileCategory::Formations => "Votre profil ne contient aucune formation",
            ProfileCategory::Competences => "Votre profil ne contient aucune compétence",
            ProfileCategory::Langues => "Votre profil ne contient aucune langue",
            ProfileCategory::Permis => "Votre profil ne précise pas si vous avez le permis de conduire",
        };
        format!("{reason} : la partie « {} » de votre CV de référence a été retirée du CV.", self.french_name())
    }
}

/// Catégories affichées par l'exemple de la référence mais vides dans le
/// profil : leur unité est retirée du CV, avec un avertissement.
///
/// # Arguments
///
/// * `example` - Données d'exemple du CV de référence.
/// * `profile` - Profil complet du candidat.
///
/// # Returns
///
/// Les catégories concernées, dans l'ordre de `ProfileCategory::all()`.
pub fn missing_categories(example: &CvData, profile: &ProfileDto) -> Vec<ProfileCategory> {
    ProfileCategory::all()
        .into_iter()
        .filter(|c| c.shown_in_example(example) && c.is_empty_in_profile(profile))
        .collect()
}

/// Champs du CV pour le prompt de l'étape A : liste fermée générée depuis
/// les catégories du profil, plus la prose, les libellés et les périodes.
///
/// # Returns
///
/// Le texte « Champs du CV » du prompt.
pub fn cv_data_fields_prompt() -> String {
    let categories: Vec<String> = ProfileCategory::all()
        .iter()
        .map(|c| format!("- {} : {}", c.schema_paths().join(", "), c.prompt_description()))
        .collect();
    format!(
        "Champs du CV, liste fermée :\n{}\n- summary : texte de la section profil.\n\
- period : la période telle qu'affichée. period_style indique pour chaque section si les périodes affichent le mois (\"month\") ou seulement l'année (\"year\") ; \"month\" pour une section absente.\n\
- labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original. labels.permis : le texte affiché pour le permis.\n\
- labels.months_short : les 12 abréviations de mois, de janvier à décembre, dans la langue et le style de l'original (\"01\"…\"12\" si l'original écrit les mois en chiffres).\n\
- labels.present : le mot qui termine une période en cours dans la langue de l'original (par exemple \"aujourd'hui\"), même s'il n'apparaît pas.\n\
Toute ligne de l'original qui ne relève d'aucune de ces catégories (par exemple une ligne d'intérêts ou de loisirs) : ne la range dans aucun champ ; elle sera un texte de la référence.",
        categories.join("\n")
    )
}
