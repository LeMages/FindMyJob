//! Formatage des périodes (expériences, projets, formations) à partir des
//! dates du profil, selon le format déclaré par la référence (`period` de
//! sa métadonnée) : granularité par section, forme du mois, séparateur.
//! Les noms de mois et « présent » viennent de `docgen::month_names`.

// Importation des crates externes
use chrono::{Datelike, NaiveDate};

// Importation des modules internes
use crate::docgen::month_names::{long_month, present, short_month};
use crate::models::{MonthStyle, PeriodGranularity, PeriodSpec};

/// Tirets interdits dans un document généré, remplacés par « - » dans un
/// séparateur déclaré.
const FORBIDDEN_DASHES: [char; 2] = ['\u{2013}', '\u{2014}'];

/// Format des périodes d'un document, dans sa langue.
///
/// # Attributes
///
/// * `month_style` - Forme du mois.
/// * `numeric_separator` - Entre le mois et l'année en forme numérique.
/// * `separator` - Entre début et fin, sans tiret cadratin ni demi-cadratin.
/// * `lang` - Langue du document (noms de mois, « présent »).
#[derive(Debug, Clone, PartialEq)]
pub struct PeriodFormat {
    pub month_style: MonthStyle,
    pub numeric_separator: String,
    pub separator: String,
    pub lang: String,
}

impl PeriodFormat {
    /// Format déclaré par une référence, dans la langue du document.
    ///
    /// # Arguments
    ///
    /// * `spec` - Format déclaré (`period` de la métadonnée).
    /// * `lang` - Langue du document.
    pub fn new(spec: &PeriodSpec, lang: &str) -> Self {
        Self {
            month_style: spec.month_style,
            numeric_separator: spec.numeric_separator.clone(),
            separator: spec.separator.replace(FORBIDDEN_DASHES, "-"),
            lang: lang.to_string(),
        }
    }
}

/// Formate une date seule selon la granularité.
fn format_date(date: NaiveDate, granularity: PeriodGranularity, format: &PeriodFormat) -> String {
    let month0 = date.month0() as usize;
    match (granularity, format.month_style) {
        (PeriodGranularity::Year, _) => date.year().to_string(),
        (PeriodGranularity::Month, MonthStyle::Short) => format!("{} {}", short_month(month0, &format.lang), date.year()),
        (PeriodGranularity::Month, MonthStyle::Long) => format!("{} {}", long_month(month0, &format.lang), date.year()),
        (PeriodGranularity::Month, MonthStyle::Numeric) => {
            format!("{:02}{}{}", date.month(), format.numeric_separator, date.year())
        }
    }
}

/// Formate une période.
///
/// - début et fin : « début - fin », ou une seule date si elles s'affichent
///   identiques (même année en granularité année, même mois sinon) ;
/// - début seul (période en cours) : « début - présent » ;
/// - fin seule : la fin ; aucune date : chaîne vide.
///
/// # Arguments
///
/// * `start` - Date de début.
/// * `end` - Date de fin, `None` si en cours.
/// * `granularity` - Mois ou année.
/// * `format` - Forme du mois, séparateur et langue.
///
/// # Returns
///
/// Période formatée.
pub fn format_period(
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    granularity: PeriodGranularity,
    format: &PeriodFormat,
) -> String {
    let start_text = start.map(|d| format_date(d, granularity, format));
    let end_text = end.map(|d| format_date(d, granularity, format));
    match (start_text, end_text) {
        (Some(s), Some(e)) if s == e => s,
        (Some(s), Some(e)) => format!("{s}{}{e}", format.separator),
        (Some(s), None) => format!("{s}{}{}", format.separator, present(&format.lang)),
        (None, Some(e)) => e,
        (None, None) => String::new(),
    }
}
