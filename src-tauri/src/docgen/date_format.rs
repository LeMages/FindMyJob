//! Formatage de la date d'une lettre selon la langue du document.

// Importation des crates externes
use chrono::{Datelike, NaiveDate};

// Importation des modules internes
use crate::docgen::month_names::long_month;

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
/// Date formatée, sans préfixe (« le » est un texte du template).
pub fn format_long_date(date: NaiveDate, lang: &str) -> String {
    let month = long_month(date.month0() as usize, lang);
    // « 1er » est l'usage français pour le premier jour du mois.
    let day = if lang != "en" && date.day() == 1 { "1er".to_string() } else { date.day().to_string() };
    format!("{day} {month} {}", date.year())
}
