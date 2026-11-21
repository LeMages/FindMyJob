//! Tests du formatage des périodes et des dates
//! (`findmyjob_lib::docgen::period_format`, `date_format`).

// Importation des modules internes
use crate::common::date;
use findmyjob_lib::docgen::date_format::format_long_date;
use findmyjob_lib::docgen::period_format::{format_period, PeriodFormat};
use findmyjob_lib::models::PeriodGranularity::{Month, Year};
use findmyjob_lib::models::{MonthStyle, PeriodSpec};

/// Format déclaré par une référence, dans une langue.
fn format(month_style: MonthStyle, separator: &str, lang: &str) -> PeriodFormat {
    let spec = PeriodSpec {
        separator: separator.to_string(),
        month_style,
        numeric_separator: ".".to_string(),
        ..Default::default()
    };
    PeriodFormat::new(&spec, lang)
}

/// Format courant : mois abrégé, en français.
fn short_fr() -> PeriodFormat {
    format(MonthStyle::Short, " - ", "fr")
}

#[test]
fn periode_au_mois() {
    let p = format_period(Some(date(2025, 11, 3)), Some(date(2026, 4, 30)), Month, &short_fr());
    assert_eq!(p, "Nov. 2025 - Avr. 2026");
}

#[test]
fn periode_a_l_annee() {
    let p = format_period(Some(date(2022, 9, 1)), Some(date(2027, 6, 30)), Year, &short_fr());
    assert_eq!(p, "2022 - 2027");
}

#[test]
fn meme_annee_affichee_une_fois() {
    let p = format_period(Some(date(2025, 1, 1)), Some(date(2025, 6, 1)), Year, &short_fr());
    assert_eq!(p, "2025");
}

#[test]
fn periode_en_cours_selon_la_langue() {
    assert_eq!(format_period(Some(date(2025, 11, 1)), None, Month, &short_fr()), "Nov. 2025 - présent");
    let en = format(MonthStyle::Short, " - ", "en");
    assert_eq!(format_period(Some(date(2025, 11, 1)), None, Month, &en), "Nov 2025 - present");
}

#[test]
fn dates_absentes() {
    assert_eq!(format_period(None, Some(date(2022, 7, 1)), Year, &short_fr()), "2022");
    assert_eq!(format_period(None, None, Month, &short_fr()), "");
}

#[test]
fn formes_du_mois() {
    let (start, end) = (Some(date(2025, 9, 1)), Some(date(2026, 2, 28)));
    assert_eq!(format_period(start, end, Month, &format(MonthStyle::Numeric, " - ", "fr")), "09.2025 - 02.2026");
    assert_eq!(format_period(start, end, Month, &format(MonthStyle::Long, " à ", "fr")), "septembre 2025 à février 2026");
    assert_eq!(format_period(start, end, Month, &format(MonthStyle::Long, " - ", "en")), "September 2025 - February 2026");
}

#[test]
fn tirets_du_separateur_normalises() {
    let p = format_period(Some(date(2022, 1, 1)), Some(date(2027, 1, 1)), Year, &format(MonthStyle::Short, " – ", "fr"));
    assert_eq!(p, "2022 - 2027");
    let p = format_period(Some(date(2022, 1, 1)), Some(date(2027, 1, 1)), Year, &format(MonthStyle::Short, " — ", "fr"));
    assert_eq!(p, "2022 - 2027");
}

#[test]
fn date_longue_selon_la_langue() {
    assert_eq!(format_long_date(date(2026, 10, 9), "fr"), "9 octobre 2026");
    assert_eq!(format_long_date(date(2026, 10, 1), "fr"), "1er octobre 2026");
    assert_eq!(format_long_date(date(2026, 10, 1), "en"), "1 October 2026");
}
