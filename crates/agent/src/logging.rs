//! Journalisation minimale des erreurs survenant après un démarrage réussi
//! (ex: base de données injoignable en cours de run). Une gestion plus
//! complète des logs (rotation, niveaux, remontée côté desktop...) est
//! prévue pour une étape ultérieure — ce module se contente d'un fichier
//! plat à côté de l'exécutable.

// Importation des modules standard
use std::fs::OpenOptions;
use std::io::Write;

// Importation des crates externes
use chrono::Utc;

const LOG_FILE: &str = "agent.log";

/// Ajoute une ligne d'erreur horodatée dans `agent.log`.
///
/// N'échoue jamais bruyamment : si l'écriture du fichier de log elle-même
/// échoue, l'erreur est simplement affichée sur stderr plutôt que de faire
/// planter l'agent.
///
/// # Arguments
///
/// * `context` - Description courte de l'opération qui a échoué.
/// * `err` - Erreur à journaliser.
pub fn log_error_to_file(context: &str, err: &dyn std::error::Error) {
    let line = format!("{} - {context}: {err}\n", Utc::now().to_rfc3339());

    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .and_then(|mut file| file.write_all(line.as_bytes()));

    if let Err(write_err) = result {
        eprintln!("Impossible d'écrire dans {LOG_FILE}: {write_err}");
    }
}
