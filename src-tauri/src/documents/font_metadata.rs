//! Lecture des familles et variantes d'un ensemble de fichiers de police,
//! d'après leurs métadonnées (table `name`, `OS/2`, axes de variation) et non
//! d'après le nom de leur dossier : c'est ce nom que Typst attend dans `font:`.

// Importation des modules standard
use std::collections::BTreeMap;
use std::path::Path;

// Importation des crates externes
use typst_library::text::{FontInfo, FontStyle, FontWeight, StandardAxes};

// Importation des modules internes
use crate::models::FontFamilyInfo;
use crate::trace_log;

/// Libellé d'une variante : graisse (ou plage de graisses d'une police
/// variable), puis style s'il n'est pas droit.
fn variant_label(info: &FontInfo) -> String {
    let weight = match StandardAxes::parse(&info.axes).wght {
        Some(axis) => {
            let (min, max) = (FontWeight::from_wght(axis.min).to_number(), FontWeight::from_wght(axis.max).to_number());
            if min == max {
                min.to_string()
            } else {
                format!("{min}–{max}")
            }
        }
        None => info.variant.weight.to_number().to_string(),
    };
    match info.variant.style {
        FontStyle::Normal => weight,
        FontStyle::Italic => format!("{weight} italique"),
        FontStyle::Oblique => format!("{weight} oblique"),
    }
}

/// Lit les familles et variantes contenues dans des fichiers de police.
///
/// Une collection (`.ttc`, `.otc`) donne une entrée par police qu'elle
/// contient. Un fichier illisible est ignoré et tracé : il ne fait pas
/// échouer la lecture des autres.
///
/// # Arguments
///
/// * `root` - Dossier auquel les chemins sont relatifs.
/// * `files` - Chemins relatifs des fichiers de police.
///
/// # Returns
///
/// Familles triées par nom, variantes triées et sans doublon.
pub fn read_families(root: &Path, files: &[impl AsRef<Path>]) -> Vec<FontFamilyInfo> {
    // Clé en minuscules : Typst compare les familles sans tenir compte de la casse.
    let mut families: BTreeMap<String, FontFamilyInfo> = BTreeMap::new();
    for file in files {
        let path = root.join(file);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                trace_log::log(&format!("[polices] lecture de {} impossible : {e}", path.display()));
                continue;
            }
        };
        let infos: Vec<FontInfo> = FontInfo::iter(&bytes).collect();
        if infos.is_empty() {
            trace_log::log(&format!("[polices] {} n'est pas une police lisible", path.display()));
        }
        for info in infos {
            let entry = families
                .entry(info.family.to_lowercase())
                .or_insert_with(|| FontFamilyInfo { name: info.family.clone(), variants: Vec::new() });
            entry.variants.push(variant_label(&info));
        }
    }

    families
        .into_values()
        .map(|mut family| {
            family.variants.sort_by_key(|label| variant_sort_key(label));
            family.variants.dedup();
            family
        })
        .collect()
}

/// Clé de tri d'un libellé de variante : graisse numérique, puis libellé.
fn variant_sort_key(label: &str) -> (u16, String) {
    let weight = label.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0);
    (weight, label.to_string())
}
