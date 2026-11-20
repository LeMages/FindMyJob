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

/// Champs optionnels des données d'exemple : vides quand l'original ne les
/// affiche pas dans un champ distinct (jamais extraits d'une puce ou d'une
/// autre ligne). Le prompt de l'étape A le dit pour chacun, et le retour
/// d'une valeur absente de l'original propose alors la chaîne vide.
pub const OPTIONAL_FIELD_NAMES: [&str; 5] = ["stack", "url", "lieu", "pour_qui", "title"];

/// Indique si un chemin de données d'exemple désigne un champ optionnel.
///
/// # Arguments
///
/// * `path` - Chemin du champ (`projects[0].stack`, `identity.title`…).
///
/// # Returns
///
/// Vrai si le dernier segment du chemin est dans `OPTIONAL_FIELD_NAMES`.
pub fn is_optional_field(path: &str) -> bool {
    let last = path.rsplit('.').next().unwrap_or(path);
    OPTIONAL_FIELD_NAMES.contains(&last)
}

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
                "une entrée par expérience professionnelle, avec ses bullets (`bullets`) et son intro (`intro`). experiences.lieu : lieu affiché dans un champ distinct de l'entreprise, chaîne vide si l'original ne l'affiche pas dans un champ distinct."
            }
            ProfileCategory::Projets => {
                "une entrée par projet, avec ses bullets et son intro. `pour_qui` est l'organisation affichée pour un projet, chaîne vide si l'original ne l'affiche pas dans un champ distinct. projects.stack et projects.url : technologies et lien affichés dans une ligne ou un champ distinct du projet ; chaîne vide si l'original n'en affiche pas. Les technologies citées dans les bullets restent dans les bullets : ne les recopie jamais dans stack."
            }
            ProfileCategory::Formations => {
                "une entrée par formation ; `details` reçoit ses lignes de détail (spécialité, cours, échange). `intro` reçoit le texte affiché avant elles."
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
- intro, bullets, details : la description d'une entrée. bullets (ou details) : ses éléments de liste, un par élément, sans leur symbole de tête (« • », « - »). intro : le texte de la description affiché avant ses éléments, sans symbole, chaîne vide s'il n'y en a pas. Une description sans aucun symbole affiche chaque ligne comme un élément de bullets (ou details).\n\
- list_marker : le symbole affiché devant chaque élément de liste des descriptions (par exemple \"•\"), tel quel ; chaîne vide si aucun élément n'en a. Jamais un séparateur de l'en-tête ou des coordonnées.\n\
- period : la période telle qu'affichée. period_style indique pour chaque section si les périodes affichent le mois (\"month\") ou seulement l'année (\"year\") ; \"month\" pour une section absente.\n\
- labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original. labels.permis : le texte affiché pour le permis.\n\
- labels.months_short : les 12 abréviations de mois, de janvier à décembre, dans la langue et le style de l'original (\"01\"…\"12\" si l'original écrit les mois en chiffres).\n\
- labels.present : le mot qui termine une période en cours dans la langue de l'original (par exemple \"aujourd'hui\"), même s'il n'apparaît pas.\n\
Toute ligne de l'original qui ne relève d'aucune de ces catégories (par exemple une ligne d'intérêts ou de loisirs) : ne la range dans aucun champ ; elle sera un texte de la référence.",
        categories.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_champs_optionnels_vides_s_ils_ne_sont_pas_affiches_a_part() {
        // Garde-fou : ces clauses ont déjà disparu une fois du prompt sans
        // qu'aucun test ne le voie (le faux LLM ignore le texte du prompt).
        let prompt = cv_data_fields_prompt();
        for clause in [
            "projects.stack et projects.url : technologies et lien affichés dans une ligne ou un champ distinct du projet ; chaîne vide si l'original n'en affiche pas.",
            "Les technologies citées dans les bullets restent dans les bullets : ne les recopie jamais dans stack.",
            "`pour_qui` est l'organisation affichée pour un projet, chaîne vide si l'original ne l'affiche pas dans un champ distinct.",
            "experiences.lieu : lieu affiché dans un champ distinct de l'entreprise, chaîne vide si l'original ne l'affiche pas dans un champ distinct.",
            "identity.title : sous-titre professionnel affiché sous le nom, chaîne vide si l'original n'en a pas.",
        ] {
            assert!(prompt.contains(clause), "clause absente : {clause}\n{prompt}");
        }
        for name in OPTIONAL_FIELD_NAMES {
            assert!(prompt.contains(name), "champ optionnel absent du prompt : {name}");
        }
    }

    #[test]
    fn prompt_garde_les_consignes_de_l_ancienne_liste() {
        let prompt = cv_data_fields_prompt();
        for clause in [
            "une entrée par ligne de compétences, la catégorie puis les compétences.",
            "`details` reçoit ses lignes de détail (spécialité, cours, échange).",
            "nom et niveau de chaque langue",
            "- summary : texte de la section profil.",
            "- period : la période telle qu'affichée.",
            "\"month\" pour une section absente.",
            "labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original.",
            "labels.permis : le texte affiché pour le permis.",
            "labels.months_short : les 12 abréviations de mois",
            "labels.present : le mot qui termine une période en cours",
        ] {
            assert!(prompt.contains(clause), "consigne absente : {clause}\n{prompt}");
        }
    }

    #[test]
    fn champ_optionnel_reconnu_par_le_dernier_segment() {
        assert!(is_optional_field("projects[0].stack"));
        assert!(is_optional_field("experiences[2].lieu"));
        assert!(is_optional_field("identity.title"));
        assert!(!is_optional_field("projects[0].titre"));
        assert!(!is_optional_field("identity.telephone"));
    }
}
