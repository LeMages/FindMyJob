//! Messages d'erreur lisibles pour l'utilisateur : une erreur technique de
//! l'API Claude (code HTTP, corps JSON, délai dépassé…) devient une phrase
//! qui dit ce qui s'est passé et quoi faire. Le détail technique est écrit
//! dans les traces, jamais montré tel quel.

// Importation des modules internes
use crate::error::AppError;
use crate::trace_log;

/// Motifs d'une erreur technique et message correspondant, dans l'ordre de
/// recherche.
const LLM_MESSAGES: [(&[&str], &str); 6] = [
    (
        &["erreur 401", "authentication_error", "invalid x-api-key"],
        "La clé API Anthropic est refusée. Vérifiez-la dans les Paramètres.",
    ),
    (&["erreur 429", "rate_limit"], "La limite de requêtes de l'API Claude est atteinte. Réessayez dans quelques minutes."),
    (
        &["erreur 529", "erreur 500", "erreur 502", "erreur 503", "overloaded"],
        "L'API Claude est momentanément indisponible ou surchargée. Réessayez dans quelques minutes.",
    ),
    (
        &["délai imparti", "Erreur réseau"],
        "Impossible de joindre l'API Claude. Vérifiez votre connexion internet puis réessayez.",
    ),
    (&["max_tokens", "tronquée"], "La réponse de l'IA a été coupée avant la fin. Réessayez."),
    (&["refusé de traiter"], "L'IA a refusé de traiter cette demande. Vérifiez le contenu fourni puis réessayez."),
];

/// Rend une erreur lisible pour l'utilisateur. Seules les erreurs de l'API
/// Claude (`AppError::Llm`) sont reformulées : les autres sont déjà écrites
/// pour l'utilisateur. L'erreur d'origine est tracée.
///
/// # Arguments
///
/// * `error` - Erreur d'origine.
/// * `action` - Ce qui a échoué, en début de phrase (ex. « La génération des documents »).
///
/// # Returns
///
/// L'erreur, reformulée si besoin, de même nature.
pub fn readable(error: AppError, action: &str) -> AppError {
    trace_log::log(&format!("[erreur] {action} : {error}"));
    let AppError::Llm(technical) = &error else {
        return error;
    };
    let message = LLM_MESSAGES
        .iter()
        .find(|(patterns, _)| patterns.iter().any(|p| technical.contains(p)))
        .map(|(_, message)| (*message).to_string())
        .unwrap_or_else(|| {
            format!(
                "{action} a échoué : l'IA n'a pas produit de réponse exploitable. Réessayez ; le détail est dans le \
                 journal de l'application."
            )
        });
    AppError::Llm(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erreur_http_reformulee() {
        let error = readable(AppError::Llm("API Claude erreur 529 Overloaded (request_id=None): {..}".into()), "L'import");
        assert!(error.to_string().starts_with("L'API Claude est momentanément indisponible"), "{error}");
    }

    #[test]
    fn erreur_inconnue_nomme_l_action() {
        let error = readable(AppError::Llm("La rédaction (CV) reste invalide après 3 essais".into()), "La génération");
        assert!(error.to_string().starts_with("La génération a échoué"), "{error}");
    }

    #[test]
    fn erreur_deja_lisible_inchangee() {
        let error = readable(AppError::Validation("Le balisage…".into()), "L'import");
        assert_eq!(error.to_string(), "Le balisage…");
    }
}
