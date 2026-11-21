//! Chiffres clés, entonnoir et répartition par issue des candidatures
//! envoyées. Logique pure.

// Importation des modules internes
use crate::models::OutcomeCount;
use crate::services::session_stats::timeline::{days_between, SentCandidature};

/// Chiffres de l'entonnoir : envoyées → réponse → entretien → offre.
pub struct FunnelCounts {
    pub sent: u32,
    pub replied: u32,
    pub interviewed: u32,
    pub offers: u32,
}

/// Issues dans l'ordre d'affichage, avec les statuts qu'elles regroupent.
const OUTCOMES: [(&str, &[&str]); 6] = [
    ("in_progress", &["a_postuler", "postule", "relance", "entretien", "entretien_passe"]),
    ("offer", &["offre", "accepte"]),
    ("refus", &["refus"]),
    ("ghosting", &["ghosting"]),
    ("abandon", &["abandon"]),
    ("offre_refusee", &["offre_refusee"]),
];

/// Compte les candidatures à chaque étape de l'entonnoir.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Nombre d'envoyées, de répondues, avec entretien et avec offre.
pub fn funnel_counts(sent: &[SentCandidature]) -> FunnelCounts {
    let count = |pred: &dyn Fn(&SentCandidature) -> bool| sent.iter().filter(|s| pred(s)).count() as u32;
    FunnelCounts {
        sent: sent.len() as u32,
        replied: count(&|s| s.first_reply_at.is_some()),
        interviewed: count(&|s| s.interviewed),
        offers: count(&|s| s.has_offer),
    }
}

/// Délai moyen entre l'envoi et la première réponse.
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Moyenne en jours, `None` sans aucune réponse.
pub fn avg_first_reply_days(sent: &[SentCandidature]) -> Option<f64> {
    mean(sent.iter().filter_map(|s| s.first_reply_at.map(|r| days_between(s.sent_at, r))))
}

/// Répartit les candidatures envoyées par issue (statut actuel).
///
/// # Arguments
///
/// * `sent` - Candidatures envoyées.
///
/// # Returns
///
/// Une entrée par issue non vide, dans l'ordre de `OUTCOMES`.
pub fn outcomes(sent: &[SentCandidature]) -> Vec<OutcomeCount> {
    OUTCOMES
        .iter()
        .map(|(outcome, statuses)| OutcomeCount {
            outcome: (*outcome).to_string(),
            count: sent.iter().filter(|s| statuses.contains(&s.candidature.status.as_str())).count() as u32,
        })
        .filter(|o| o.count > 0)
        .collect()
}

/// Moyenne d'une suite de valeurs.
///
/// # Arguments
///
/// * `values` - Valeurs à moyenner.
///
/// # Returns
///
/// Moyenne arithmétique, `None` pour une suite vide.
pub fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, n) = values.fold((0.0, 0u32), |(sum, n), v| (sum + v, n + 1));
    (n > 0).then(|| sum / f64::from(n))
}
