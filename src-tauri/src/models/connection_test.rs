//! DTO du test de connexion à l'API Claude (écran Paramètres).

/// Résultat du test de connexion à l'API Claude.
///
/// # Attributes
///
/// * `success` - Vrai si l'appel a abouti.
/// * `model` - Modèle testé.
/// * `message` - Message affiché (succès ou erreur).
/// * `latency_ms` - Durée de l'appel, en millisecondes.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub model: String,
    pub message: String,
    pub latency_ms: u64,
}
