//! Tests du formatage des périodes et des dates
//! (`findmyjob_core::docgen::period_format`, `date_format`).

// Importation des modules internes
use crate::common::date;
use findmyjob_core::docgen::date_format::format_long_date;
use findmyjob_core::docgen::period_format::{detect_separator, format_period, PeriodStyleText};
use findmyjob_core::models::PeriodGranularity::{Month, Year};

/// Style français de la référence (abréviations « Nov. », « Avr. »).
fn style_fr() -> PeriodStyleText {
    let months = ["Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.", "Déc."];
    PeriodStyleText {
        months_short: months.iter().map(|m| m.to_string()).collect(),
        present: "aujourd'hui".to_string(),
        separator: " – ".to_string(),
    }
}

#[test]
fn periode_au_mois() {
    let p = format_period(Some(date(2025, 11, 3)), Some(date(2026, 4, 30)), Month, &style_fr());
    assert_eq!(p, "Nov. 2025 – Avr. 2026");
}

#[test]
fn periode_a_l_annee() {
    let p = format_period(Some(date(2022, 9, 1)), Some(date(2027, 6, 30)), Year, &style_fr());
    assert_eq!(p, "2022 – 2027");
}

#[test]
fn meme_annee_affichee_une_fois() {
    let p = format_period(Some(date(2025, 1, 1)), Some(date(2025, 6, 1)), Year, &style_fr());
    assert_eq!(p, "2025");
}

#[test]
fn periode_en_cours() {
    let p = format_period(Some(date(2025, 11, 1)), None, Month, &style_fr());
    assert_eq!(p, "Nov. 2025 – aujourd'hui");
    let sans_present = PeriodStyleText { present: String::new(), ..style_fr() };
    assert_eq!(format_period(Some(date(2025, 11, 1)), None, Month, &sans_present), "Nov. 2025");
}

#[test]
fn dates_absentes() {
    assert_eq!(format_period(None, Some(date(2022, 7, 1)), Year, &style_fr()), "2022");
    assert_eq!(format_period(None, None, Month, &style_fr()), "");
}

#[test]
fn abreviations_manquantes_retombent_sur_les_chiffres() {
    let style = PeriodStyleText { months_short: Vec::new(), ..style_fr() };
    assert_eq!(format_period(Some(date(2025, 3, 1)), None, Month, &PeriodStyleText { present: String::new(), ..style }), "03/2025");
}

#[test]
fn separateur_detecte_dans_la_reference() {
    assert_eq!(detect_separator(["2022 - 2027"]), " - ");
    assert_eq!(detect_separator(["Nov. 2025 – Avr. 2026"]), " – ");
    assert_eq!(detect_separator(["2022"]), " – ");
}

#[test]
fn date_longue_selon_la_langue() {
    assert_eq!(format_long_date(date(2026, 10, 9), "fr"), "9 octobre 2026");
    assert_eq!(format_long_date(date(2026, 10, 1), "fr"), "1er octobre 2026");
    assert_eq!(format_long_date(date(2026, 10, 1), "en"), "1 October 2026");
}
