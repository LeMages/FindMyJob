//! Contenu rédigé par le LLM pour un CV ou une lettre : uniquement les
//! champs de prose. Identité, intitulés, entreprises, dates, formations,
//! langues et destinataire sont remplis par Rust depuis le profil et
//! l'offre (voir `src-tauri/src/docgen`).

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::models::doc_data::SkillsGroup;

/// Bullets rédigés pour une entrée du profil (expérience ou projet).
///
/// # Attributes
///
/// * `id` - Identifiant de l'expérience ou du projet dans le profil.
/// * `bullets` - Bullets rédigés, dans l'ordre d'affichage.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ItemBullets {
    pub id: i64,
    pub bullets: Vec<String>,
}

/// Contenu rédigé d'un CV.
///
/// # Attributes
///
/// * `title` - Titre affiché sous le nom.
/// * `summary` - Résumé (section profil).
/// * `experiences` - Bullets de chaque expérience retenue.
/// * `projects` - Bullets de chaque projet retenu.
/// * `skills_groups` - Compétences du profil regroupées par catégorie.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CvContent {
    pub title: String,
    pub summary: String,
    pub experiences: Vec<ItemBullets>,
    pub projects: Vec<ItemBullets>,
    pub skills_groups: Vec<SkillsGroup>,
}

/// Contenu rédigé d'une lettre de motivation.
///
/// # Attributes
///
/// * `subject` - Objet de la lettre, sans son préfixe.
/// * `paragraphs` - Paragraphes du corps, entre la formule d'appel et la
///   formule de politesse finale.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LmContent {
    pub subject: String,
    pub paragraphs: Vec<String>,
}

/// Champ de prose d'un contenu rédigé, repéré par son chemin JSON.
///
/// # Attributes
///
/// * `path` - Chemin du champ (ex. `experiences[0].bullets[1]`).
/// * `text` - Texte du champ.
#[derive(Debug, Clone, PartialEq)]
pub struct ProseField {
    pub path: String,
    pub text: String,
}

impl CvContent {
    /// Liste les champs de prose du CV (titre, résumé, bullets).
    pub fn prose_fields(&self) -> Vec<ProseField> {
        let mut fields = vec![
            ProseField { path: "title".to_string(), text: self.title.clone() },
            ProseField { path: "summary".to_string(), text: self.summary.clone() },
        ];
        for (kind, items) in [("experiences", &self.experiences), ("projects", &self.projects)] {
            for (i, item) in items.iter().enumerate() {
                for (j, bullet) in item.bullets.iter().enumerate() {
                    fields.push(ProseField { path: format!("{kind}[{i}].bullets[{j}]"), text: bullet.clone() });
                }
            }
        }
        fields
    }

    /// Applique une transformation à chaque champ de prose ; elle reçoit le
    /// chemin du champ (celui de `prose_fields`) puis son texte.
    pub fn map_prose(&mut self, transform: impl Fn(&str, &str) -> String) {
        self.title = transform("title", &self.title);
        self.summary = transform("summary", &self.summary);
        for (kind, items) in [("experiences", &mut self.experiences), ("projects", &mut self.projects)] {
            for (i, item) in items.iter_mut().enumerate() {
                for (j, bullet) in item.bullets.iter_mut().enumerate() {
                    *bullet = transform(&format!("{kind}[{i}].bullets[{j}]"), bullet);
                }
            }
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
