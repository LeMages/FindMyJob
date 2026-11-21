//! Comparaisons « Ce qui marche » : taux de réponse et entretiens des
//! candidatures envoyées selon le modèle de CV, le modèle de lettre, la
//! source de l'offre et la présence d'un contact recruteur. Logique pure.

// Importation des modules standard
use std::collections::BTreeMap;

// Importation des modules internes
use crate::models::{Comparison, ComparisonRow};
use crate::services::session_stats::timeline::SentCandidature;

/// En dessous de ce nombre d'envois, une ligne est grisée « trop peu ».
pub const MIN_SENT_PER_ROW: u32 = 5;

/// Clé d'une version de référence supprimée (ou absente).
const UNKNOWN_KEY: &str = "unknown";

/// Calcule les quatre comparaisons, sans comparaison vide.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Comparaisons par modèle de CV, modèle de lettre, source et contact.
pub fn comparisons(sent: &[SentCandidature]) -> Vec<Comparison> {
    let version_key = |v: Option<i64>| v.map_or(UNKNOWN_KEY.to_string(), |n| n.to_string());
    let mut cv = group(sent, |s| version_key(s.candidature.cv_version));
    let mut lm = group(sent, |s| version_key(s.candidature.lm_version));
    sort_versions(&mut cv);
    sort_versions(&mut lm);
    let mut source = group(sent, |s| s.candidature.source.clone());
    source.sort_by(|a, b| b.sent.cmp(&a.sent).then_with(|| a.key.cmp(&b.key)));
    let mut contact = group(sent, |s| if s.candidature.has_contact { "with" } else { "without" }.to_string());
    contact.sort_by_key(|r| r.key != "with");

    [("cv_template", cv), ("lm_template", lm), ("source", source), ("contact", contact)]
        .into_iter()
        .filter(|(_, rows)| !rows.is_empty())
        .map(|(criterion, rows)| Comparison { criterion: criterion.to_string(), rows: mark_best(rows) })
        .collect()
}

/// Regroupe les candidatures par clé et compte envois, réponses, entretiens.
fn group(sent: &[SentCandidature], key_of: impl Fn(&SentCandidature) -> String) -> Vec<ComparisonRow> {
    let mut rows: BTreeMap<String, ComparisonRow> = BTreeMap::new();
    for s in sent {
        let key = key_of(s);
        let row = rows.entry(key.clone()).or_insert_with(|| ComparisonRow {
            key,
            sent: 0,
            replied: 0,
            interviewed: 0,
            low_sample: false,
            best: false,
        });
        row.sent += 1;
        row.replied += u32::from(s.first_reply_at.is_some());
        row.interviewed += u32::from(s.interviewed);
    }
    rows.into_values().collect()
}

/// Versions par numéro croissant, version inconnue en dernier.
fn sort_versions(rows: &mut [ComparisonRow]) {
    rows.sort_by_key(|r| r.key.parse::<i64>().map_or((1, 0), |n| (0, n)));
}

/// Marque les lignes trop peu fournies et la (ou les) meilleure(s) parmi les
/// autres : celles qui ont le plus haut taux de réponse.
fn mark_best(mut rows: Vec<ComparisonRow>) -> Vec<ComparisonRow> {
    let rate = |r: &ComparisonRow| f64::from(r.replied) / f64::from(r.sent.max(1));
    for row in &mut rows {
        row.low_sample = row.sent < MIN_SENT_PER_ROW;
    }
    let best = rows.iter().filter(|r| !r.low_sample).map(rate).fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))));
    if let Some(best) = best {
        for row in &mut rows {
            // Comparaison exacte voulue : les taux viennent des mêmes entiers.
            row.best = !row.low_sample && (rate(row) - best).abs() < f64::EPSILON;
        }
    }
    rows
}
