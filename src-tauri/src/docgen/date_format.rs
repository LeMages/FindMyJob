//! Formatage de la date d'une lettre selon la langue du document.

// Importation des crates externes
use chrono::{Datelike, NaiveDate};

/// Mois en toutes lettres, en français.
const MONTHS_FR: [&str; 12] = [
    "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre",
    "décembre",
];
/// Mois en toutes lettres, en anglais.
const MONTHS_EN: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];

/// Formate une date en toutes lettres : « 9 octobre 2026 » en français,
/// « 9 October 2026 » en anglais (toute autre langue retombe sur le français).
///
/// # Arguments
///
/// * `date` - Date à formater.
/// * `lang` - Code de langue du document (`"fr"`, `"en"`).
///
/// # Returns
///
/// Date formatée, sans préfixe (« le » est un libellé du template).
pub fn format_long_date(date: NaiveDate, lang: &str) -> String {
    let months = if lang == "en" { &MONTHS_EN } else { &MONTHS_FR };
    // month0() est toujours dans 0..12 : l'indexation ne peut pas paniquer.
    let month = months[date.month0() as usize];
    // « 1er » est l'usage français pour le premier jour du mois.
    let day = if lang != "en" && date.day() == 1 { "1er".to_string() } else { date.day().to_string() };
    format!("{day} {month} {}", date.year())
}
