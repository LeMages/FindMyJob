//! Textes d'une référence qui ne viennent ni du profil ni de la rédaction :
//! textes visibles écrits en dur (« Références disponibles sur demande »,
//! « P.J. : CV »…), lus par le template dans `data.texts.<id>`, et consignes
//! de rédaction (consignes générales, zones `[FIXE]` / `[VARIABLE]` /
//! `[ADAPTABLE]`). La langue de ces textes est portée explicitement : c'est
//! l'unité traduite quand un document est généré dans une autre langue.
//!
//! Stockés dans `reference_texts.json`, à côté du template de la version.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des crates externes
use serde::{Deserialize, Serialize};

// Importation des modules internes
use crate::models::prose_guidance::ProseGuidance;

/// Version du format des fichiers d'une référence templatisée. Une version
/// convertie avec un autre format doit être réimportée.
pub const REFERENCE_FORMAT_VERSION: u32 = 2;

/// Nom du fichier des textes de la référence, dans le dossier de la version.
pub const REFERENCE_TEXTS_FILE: &str = "reference_texts.json";

/// Textes et consignes d'une référence, dans une langue.
///
/// # Attributes
///
/// * `format_version` - Format de la référence templatisée
///   (`REFERENCE_FORMAT_VERSION`).
/// * `lang` - Langue de ces textes (`"fr"` ou `"en"`).
/// * `texts` - Textes en dur, par identifiant (`t1`, `t2`…), lus par le
///   template dans `data.texts`.
/// * `guidance` - Consignes de rédaction (générales et zones de prose).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ReferenceTexts {
    pub format_version: u32,
    pub lang: String,
    pub texts: BTreeMap<String, String>,
    pub guidance: ProseGuidance,
}
