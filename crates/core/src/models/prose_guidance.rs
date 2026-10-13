//! Consignes de rédaction lues dans une référence : découpage de la prose
//! (objet, paragraphes, titre, résumé) en zones balisées `[FIXE]`,
//! `[VARIABLE]` ou `[ADAPTABLE]`, avec leur consigne.
//!
//! Extraites à la templatisation (les commentaires de la référence ne sont
//! pas rendus) et rangées dans `example_data.json`, elles sont transmises à
//! la rédaction : une zone fixe est recopiée mot pour mot (seules les
//! informations du profil y sont mises à jour), une zone variable est tirée
//! de l'offre, une zone adaptable est rédigée depuis le profil. Une prose
//! sans zone est entièrement adaptable.

// Importation des crates externes
use serde::{Deserialize, Serialize};

/// Rôle d'une zone de prose.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ZoneMode {
    /// Recopiée mot pour mot ; seules les informations du profil changent.
    Fixe,
    /// Remplacée par les informations tirées de l'offre.
    Variable,
    /// Réorganisée depuis le profil selon l'offre, sans rien inventer.
    Adaptable,
}

impl ZoneMode {
    /// Balise de la zone, telle qu'écrite dans la référence.
    pub fn tag(self) -> &'static str {
        match self {
            ZoneMode::Fixe => "[FIXE]",
            ZoneMode::Variable => "[VARIABLE]",
            ZoneMode::Adaptable => "[ADAPTABLE]",
        }
    }
}

/// Zone d'un champ de prose.
///
/// # Attributes
///
/// * `field` - Champ de prose qui contient la zone : `subject`,
///   `paragraphs[i]` pour une lettre, `identity.title` ou `summary` pour un CV.
/// * `mode` - Rôle de la zone.
/// * `instruction` - Consigne propre à la zone, telle qu'écrite dans la
///   référence (chaîne vide si aucune).
/// * `text` - Texte de la zone dans la référence. Les zones d'un même champ,
///   mises bout à bout, redonnent exactement le champ.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProseZone {
    pub field: String,
    pub mode: ZoneMode,
    pub instruction: String,
    pub text: String,
}

/// Consignes de rédaction d'une référence.
///
/// # Attributes
///
/// * `general` - Consignes générales de la référence (chaîne vide si aucune).
/// * `zones` - Zones balisées, dans l'ordre du document.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ProseGuidance {
    pub general: String,
    pub zones: Vec<ProseZone>,
}

impl ProseGuidance {
    /// Zones d'un champ de prose, dans l'ordre.
    pub fn zones_of<'a>(&'a self, field: &'a str) -> impl Iterator<Item = &'a ProseZone> + 'a {
        self.zones.iter().filter(move |zone| zone.field == field)
    }

    /// Textes des zones fixes d'un champ.
    pub fn fixed_texts<'a>(&'a self, field: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.zones_of(field).filter(|zone| zone.mode == ZoneMode::Fixe).map(|zone| zone.text.as_str())
    }
}
