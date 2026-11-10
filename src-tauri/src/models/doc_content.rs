//! Contenu rédigé par le LLM pour un CV ou une lettre : uniquement les
//! champs de prose. Identité, intitulés, entreprises, dates, descriptions
//! des expériences, projets et formations, langues et destinataire sont
//! remplis par Rust depuis le profil et l'offre (voir `src-tauri/src/docgen`).

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::models::doc_data::SkillsGroup;

/// Contenu rédigé d'un CV.
///
/// # Attributes
///
/// * `title` - Titre affiché sous le nom.
/// * `summary` - Résumé (section profil).
/// * `skills_groups` - Compétences du profil regroupées par catégorie.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CvContent {
    pub title: String,
    pub summary: String,
    pub skills_groups: Vec<SkillsGroup>,
}

/// Contenu rédigé d'une lettre de motivation.
///
/// # Attributes
///
/// * `subject` - Objet de la lettre, sans son préfixe.
/// * `paragraphs` - Paragraphes du corps, entre la formule d'appel et la
///   formule de politesse finale.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LmContent {
    pub subject: String,
    pub paragraphs: Vec<String>,
}

/// Champ de prose d'un contenu rédigé, repéré par son chemin JSON.
///
/// # Attributes
///
/// * `path` - Chemin du champ (ex. `paragraphs[1]`).
/// * `text` - Texte du champ.
#[derive(Debug, Clone, PartialEq)]
pub struct ProseField {
    pub path: String,
    pub text: String,
}

impl CvContent {
    /// Liste les champs de prose du CV (titre, résumé, catégories des
    /// groupes de compétences).
    pub fn prose_fields(&self) -> Vec<ProseField> {
        let mut fields = vec![
            ProseField { path: "title".to_string(), text: self.title.clone() },
            ProseField { path: "summary".to_string(), text: self.summary.clone() },
        ];
        for (i, group) in self.skills_groups.iter().enumerate() {
            fields.push(ProseField { path: format!("skills_groups[{i}].category"), text: group.category.clone() });
        }
        fields
    }

    /// Applique une transformation à chaque champ de prose ; elle reçoit le
    /// chemin du champ (celui de `prose_fields`) puis son texte.
    pub fn map_prose(&mut self, transform: impl Fn(&str, &str) -> String) {
        self.title = transform("title", &self.title);
        self.summary = transform("summary", &self.summary);
        for (i, group) in self.skills_groups.iter_mut().enumerate() {
            group.category = transform(&format!("skills_groups[{i}].category"), &group.category);
        }
    }
}

impl LmContent {
    /// Liste les champs de prose de la lettre (objet, paragraphes).
    pub fn prose_fields(&self) -> Vec<ProseField> {
        let mut fields = vec![ProseField { path: "subject".to_string(), text: self.subject.clone() }];
        for (i, paragraph) in self.paragraphs.iter().enumerate() {
            fields.push(ProseField { path: format!("paragraphs[{i}]"), text: paragraph.clone() });
        }
        fields
    }

    /// Applique une transformation à chaque champ de prose ; elle reçoit le
    /// chemin du champ (celui de `prose_fields`) puis son texte.
    pub fn map_prose(&mut self, transform: impl Fn(&str, &str) -> String) {
        self.subject = transform("subject", &self.subject);
        for (i, paragraph) in self.paragraphs.iter_mut().enumerate() {
            *paragraph = transform(&format!("paragraphs[{i}]"), paragraph);
        }
    }
}
