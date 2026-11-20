//! DTOs des statistiques d'une session (onglet Statistiques du suivi) et
//! données brutes lues en base pour les calculer.

// Importation des crates externes
use chrono::{DateTime, NaiveDate, Utc};

// ======================================================================
// Données brutes (lues par `db::session_stats`, jamais envoyées au frontend)
// ======================================================================

/// Candidature d'une session, réduite aux champs utiles aux statistiques.
pub struct StatsCandidature {
    pub id: i64,
    pub status: String,
    pub source: String,
    /// Numéro de la version du CV de référence utilisée, `None` si la version
    /// a été supprimée ou si la candidature n'a pas de document.
    pub cv_version: Option<i64>,
    /// Numéro de la version de la lettre de référence utilisée.
    pub lm_version: Option<i64>,
    /// Au moins un contact est lié à la candidature.
    pub has_contact: bool,
}

/// Événement de l'historique d'une candidature.
pub struct StatsEvent {
    pub candidature_id: i64,
    pub event_type: String,
    pub changed_at: DateTime<Utc>,
}

/// Entretien d'une candidature.
pub struct StatsInterview {
    pub candidature_id: i64,
    pub type_entretien: Option<String>,
    pub date_entretien: Option<DateTime<Utc>>,
    pub issue: String,
}

/// Données brutes d'une session, entrée du calcul des statistiques.
pub struct StatsInput {
    pub candidatures: Vec<StatsCandidature>,
    /// Événements de toutes les candidatures, triés par date croissante.
    pub events: Vec<StatsEvent>,
    pub interviews: Vec<StatsInterview>,
}

// ======================================================================
// DTO envoyé au frontend
// ======================================================================

/// Statistiques d'une session, calculées sur les candidatures envoyées.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionStats {
    /// Assez de candidatures envoyées pour afficher les statistiques.
    pub enough_data: bool,
    /// Nombre d'envois à partir duquel les statistiques s'affichent.
    pub min_sent: u32,
    /// Nombre d'envois sous lequel une ligne de comparaison est « trop peu ».
    pub min_sent_per_row: u32,
    /// Candidatures envoyées (avec un événement `candidature_envoyee`).
    pub sent: u32,
    /// Candidatures envoyées depuis le début de la semaine en cours.
    pub sent_this_week: u32,
    /// Candidatures envoyées ayant reçu au moins une réponse.
    pub replied: u32,
    /// Candidatures envoyées ayant eu au moins un entretien.
    pub interviewed: u32,
    /// Candidatures envoyées ayant reçu une offre.
    pub offers: u32,
    /// Délai moyen (jours) entre l'envoi et la première réponse.
    pub avg_first_reply_days: Option<f64>,
    /// Répartition des candidatures envoyées par issue, sans issue vide.
    pub outcomes: Vec<OutcomeCount>,
    /// Activité des dernières semaines, de la plus ancienne à la plus récente.
    pub weeks: Vec<WeekActivity>,
    /// Comparaisons « Ce qui marche », sans comparaison vide.
    pub comparisons: Vec<Comparison>,
    /// Entretiens passés par type, sans type vide.
    pub interview_types: Vec<InterviewTypeCount>,
    /// Durées médianes depuis l'envoi.
    pub median_days: MedianDurations,
    /// Effet des relances.
    pub follow_ups: FollowUpStats,
}

/// Nombre de candidatures envoyées pour une issue.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutcomeCount {
    /// `in_progress`, `offer`, `refus`, `ghosting`, `abandon` ou `offre_refusee`.
    pub outcome: String,
    pub count: u32,
}

/// Activité d'une semaine (du lundi au dimanche).
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WeekActivity {
    /// Lundi de la semaine.
    pub week_start: NaiveDate,
    /// Candidatures envoyées dans la semaine.
    pub sent: u32,
    /// Entretiens datés dans la semaine.
    pub interviews: u32,
}

/// Comparaison des candidatures envoyées selon un critère.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Comparison {
    /// `cv_template`, `lm_template`, `source` ou `contact`.
    pub criterion: String,
    pub rows: Vec<ComparisonRow>,
}

/// Une valeur du critère comparé.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComparisonRow {
    /// Numéro de version (`"3"`), valeur de `source`, `with` / `without` pour
    /// le contact, ou `unknown` (version supprimée).
    pub key: String,
    pub sent: u32,
    pub replied: u32,
    pub interviewed: u32,
    /// Trop peu d'envois pour conclure.
    pub low_sample: bool,
    /// Meilleur taux de réponse parmi les lignes assez fournies.
    pub best: bool,
}

/// Entretiens passés d'un type.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InterviewTypeCount {
    /// Type d'entretien (`autre` quand il n'est pas renseigné).
    pub interview_type: String,
    pub passed: u32,
    pub positive: u32,
}

/// Durées médianes (jours) entre l'envoi et une issue.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MedianDurations {
    pub to_offer: Option<f64>,
    pub to_rejection: Option<f64>,
    pub to_ghosting: Option<f64>,
}

/// Relances envoyées et leur effet.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FollowUpStats {
    pub sent: u32,
    /// Relances suivies d'une réponse.
    pub answered: u32,
    /// Délai moyen (jours) entre l'envoi et la première relance.
    pub avg_days_before: Option<f64>,
}
