//! Noms de mois et borne « présent » des documents, en français et en
//! anglais : la seule table utilisée par les dates des lettres et les
//! périodes des CV. Toute autre langue retombe sur le français.

/// Mois en toutes lettres, en français.
const LONG_FR: [&str; 12] = [
    "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre",
    "décembre",
];
/// Mois en toutes lettres, en anglais.
const LONG_EN: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];
/// Mois abrégés, en français (abréviations usuelles). La majuscule vient
/// de leur place : une période commence par son mois.
const SHORT_FR: [&str; 12] =
    ["Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.", "Déc."];
/// Mois abrégés, en anglais.
const SHORT_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/// Borne de fin d'une période en cours, en français.
const PRESENT_FR: &str = "présent";
/// Borne de fin d'une période en cours, en anglais.
const PRESENT_EN: &str = "present";

/// Nom complet d'un mois.
///
/// # Arguments
///
/// * `month0` - Mois, de 0 (janvier) à 11.
/// * `lang` - Code de langue du document.
///
/// # Returns
///
/// Le nom, ou une chaîne vide hors de 0..12.
pub fn long_month(month0: usize, lang: &str) -> &'static str {
    let names = if lang == "en" { &LONG_EN } else { &LONG_FR };
    names.get(month0).copied().unwrap_or_default()
}

/// Abréviation d'un mois.
///
/// # Arguments
///
/// * `month0` - Mois, de 0 (janvier) à 11.
/// * `lang` - Code de langue du document.
///
/// # Returns
///
/// L'abréviation, ou une chaîne vide hors de 0..12.
pub fn short_month(month0: usize, lang: &str) -> &'static str {
    let names = if lang == "en" { &SHORT_EN } else { &SHORT_FR };
    names.get(month0).copied().unwrap_or_default()
}

/// Borne de fin d'une période en cours (« présent », « present »).
///
/// # Arguments
///
/// * `lang` - Code de langue du document.
pub fn present(lang: &str) -> &'static str {
    if lang == "en" {
        PRESENT_EN
    } else {
        PRESENT_FR
    }
}
