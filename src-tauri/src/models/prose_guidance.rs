//! Consignes de rédaction d'une référence, transmises au rédacteur :
//! consignes générales (`guidance` de la métadonnée) et emplacements
//! rédigés (`slots`), chacun avec son mode, sa consigne et sa valeur
//! d'exemple. Un emplacement variable est tiré de l'offre, un emplacement
//! adaptable est rédigé depuis le profil. Une prose sans emplacement
//! déclaré est rédigée en mode variable.
//!
//! Les parties fixes d'un document sont du texte du template : elles ne
//! passent jamais par la rédaction.

// Importation des crates externes
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Importation des modules internes
use crate::models::reference_contract::{Slot, SlotMode};

/// Emplacement rédigé d'un champ de prose.
///
/// # Attributes
///
/// * `field` - Champ de prose : `subject`, `paragraphs[i]` pour une lettre,
///   `identity.title` ou `summary` pour un CV.
/// * `mode` - Rôle de l'emplacement.
/// * `instruction` - Consigne propre à l'emplacement (chaîne vide si aucune).
/// * `text` - Valeur d'exemple du champ dans la référence.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProseZone {
    pub field: String,
    pub mode: SlotMode,
    pub instruction: String,
    pub text: String,
}

/// Consignes de rédaction d'une référence.
///
/// # Attributes
///
/// * `general` - Consignes générales de la référence (chaîne vide si aucune).
/// * `zones` - Emplacements rédigés déclarés, dans l'ordre de la métadonnée.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ProseGuidance {
    pub general: String,
    pub zones: Vec<ProseZone>,
}

impl ProseGuidance {
    /// Consignes d'une référence, à partir de son contrat enregistré.
    ///
    /// # Arguments
    ///
    /// * `general` - Consignes générales.
    /// * `slots` - Emplacements déclarés.
    /// * `example` - Valeurs d'exemple de la référence, en JSON.
    ///
    /// # Returns
    ///
    /// `ProseGuidance`, un emplacement par slot, avec sa valeur d'exemple.
    pub fn from_slots(general: &str, slots: &[Slot], example: &Value) -> Self {
        let zones = slots
            .iter()
            .map(|slot| ProseZone {
                field: slot.field.clone(),
                mode: slot.mode,
                instruction: slot.instruction.clone(),
                text: field_text(example, &slot.field).unwrap_or_default(),
            })
            .collect();
        Self { general: general.trim().to_string(), zones }
    }

    /// Emplacements d'un champ de prose, dans l'ordre.
    pub fn zones_of<'a>(&'a self, field: &'a str) -> impl Iterator<Item = &'a ProseZone> + 'a {
        self.zones.iter().filter(move |zone| zone.field == field)
    }
}

impl SlotMode {
    /// Balise du mode, telle que citée au rédacteur.
    pub fn tag(self) -> &'static str {
        match self {
            SlotMode::Variable => "[VARIABLE]",
            SlotMode::Adaptable => "[ADAPTABLE]",
        }
    }
}

/// Texte d'un champ de prose dans les valeurs d'exemple : `summary`,
/// `identity.title`, `subject` ou `paragraphs[i]`.
fn field_text(example: &Value, field: &str) -> Option<String> {
    let mut current = example;
    for part in field.split('.') {
        current = match part.split_once('[') {
            Some((name, index)) => {
                let index: usize = index.strip_suffix(']')?.parse().ok()?;
                current.get(name)?.get(index)?
            }
            None => current.get(part)?,
        };
    }
    current.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valeur_d_exemple_de_chaque_emplacement() {
        let example = serde_json::json!({
            "identity": { "title": "Titre" },
            "paragraphs": ["Premier", "Second"],
        });
        assert_eq!(field_text(&example, "identity.title").as_deref(), Some("Titre"));
        assert_eq!(field_text(&example, "paragraphs[1]").as_deref(), Some("Second"));
        assert_eq!(field_text(&example, "paragraphs[2]"), None);
        assert_eq!(field_text(&example, "summary"), None);
    }
}
