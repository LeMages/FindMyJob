//! Formatage des périodes (expériences, projets, formations) à partir des
//! dates du profil, dans le style de la référence : granularité (mois ou
//! année), abréviations de mois et séparateur.

// Importation des crates externes
use chrono::{Datelike, NaiveDate};

// Importation des modules internes
use crate::models::PeriodGranularity;

/// Séparateur utilisé quand la référence n'en montre aucun.
const DEFAULT_SEPARATOR: &str = " – ";
/// Séparateurs reconnus dans les périodes de la référence, du plus au moins
/// spécifique.
const KNOWN_SEPARATORS: [&str; 3] = [" – ", " — ", " - "];

/// Style de formatage des périodes d'un document.
///
/// # Attributes
///
/// * `months_short` - Les 12 abréviations de mois de la référence.
/// * `present` - Borne de fin d'une période en cours (ex. « aujourd'hui »).
/// * `separator` - Séparateur entre début et fin (ex. « – »).
#[derive(Debug, Clone, PartialEq)]
pub struct PeriodStyleText {
    pub months_short: Vec<String>,
    pub present: String,
    pub separator: String,
}

/// Retrouve le séparateur de période utilisé par la référence.
///
/// # Arguments
///
/// * `periods` - Périodes telles qu'affichées dans la référence.
///
/// # Returns
///
/// Le premier séparateur connu trouvé, sinon « – ».
pub fn detect_separator<'a>(periods: impl IntoIterator<Item = &'a str>) -> String {
    let periods: Vec<&str> = periods.into_iter().collect();
    KNOWN_SEPARATORS
        .iter()
        .find(|sep| periods.iter().any(|p| p.contains(*sep)))
        .unwrap_or(&DEFAULT_SEPARATOR)
        .to_string()
}

/// Formate une date seule selon la granularité.
fn format_date(date: NaiveDate, granularity: PeriodGranularity, style: &PeriodStyleText) -> String {
    match granularity {
        PeriodGranularity::Year => date.year().to_string(),
        PeriodGranularity::Month => match style.months_short.get(date.month0() as usize) {
            Some(month) if !month.is_empty() => format!("{month} {}", date.year()),
            // Sans abréviations exploitables, le mois reste lisible en chiffres.
            _ => format!("{:02}/{}", date.month(), date.year()),
        },
    }
}

/// Formate une période.
///
/// - début et fin : « début – fin », ou une seule date si elles s'affichent
///   identiques (même année en granularité année, même mois sinon) ;
/// - début seul (période en cours) : « début – aujourd'hui », ou le début
///   seul si la référence n'a pas de libellé pour le présent ;
/// - fin seule : la fin ; aucune date : chaîne vide.
///
/// # Arguments
///
/// * `start` - Date de début.
/// * `end` - Date de fin, `None` si en cours.
/// * `granularity` - Mois ou année.
/// * `style` - Abréviations, libellé du présent et séparateur.
///
/// # Returns
///
/// Période formatée.
pub fn format_period(
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    granularity: PeriodGranularity,
    style: &PeriodStyleText,
) -> String {
    let start_text = start.map(|d| format_date(d, granularity, style));
    let end_text = end.map(|d| format_date(d, granularity, style));
    match (start_text, end_text) {
        (Some(s), Some(e)) if s == e => s,
        (Some(s), Some(e)) => format!("{s}{}{e}", style.separator),
        (Some(s), None) if style.present.is_empty() => s,
        (Some(s), None) => format!("{s}{}{}", style.separator, style.present),
        (None, Some(e)) => e,
        (None, None) => String::new(),
    }
}
