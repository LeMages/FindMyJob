//! Source unique de l'heure courante et format unique des dates stockées.
//!
//! Toutes les dates-heures écrites en base sont du texte RFC 3339 UTC à
//! précision fixe en millisecondes (`2026-10-10T08:15:30.125Z`) : longueur
//! constante, donc le tri des chaînes (`ORDER BY`, `MAX`) est le tri
//! chronologique. Les dates simples sont au format `YYYY-MM-DD`.

// Importation des crates externes
use chrono::{DateTime, NaiveDate, Utc};

/// Format des dates-heures stockées (RFC 3339 UTC, millisecondes, suffixe `Z`).
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Format des dates simples stockées.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Heure courante au format de stockage.
///
/// Une transaction lit l'heure une seule fois et transmet la même valeur à
/// toutes ses écritures : les horodatages d'une même opération sont égaux.
///
/// # Returns
///
/// Horodatage RFC 3339 UTC en millisecondes.
pub fn now_utc() -> String {
    format_timestamp(Utc::now())
}

/// Convertit une date-heure au format de stockage.
///
/// # Arguments
///
/// * `dt` - Date-heure UTC à formater (ex. date d'entretien saisie).
///
/// # Returns
///
/// Horodatage RFC 3339 UTC en millisecondes.
pub fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.format(TIMESTAMP_FORMAT).to_string()
}

/// Convertit une date simple au format de stockage.
///
/// # Arguments
///
/// * `date` - Date à formater.
///
/// # Returns
///
/// Date au format `YYYY-MM-DD`.
pub fn format_date(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}
