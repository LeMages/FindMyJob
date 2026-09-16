//! Gestion de la clé API Anthropic sur un serveur headless.
//!
//! Contrairement au desktop (trousseau OS en release, `.env` en debug), un
//! serveur n'a pas de session utilisateur OS ni de distinction debug/release
//! pertinente : la clé vit uniquement dans la variable d'environnement du
//! processus `ANTHROPIC_API_KEY`, chargée depuis `.env` au démarrage
//! (`dotenvy`, voir `main.rs`) et modifiable à chaud via l'API. Elle n'est
//! jamais persistée en base de données ni sur disque.
//!
//! `findmyjob_core::llm::client` lit directement `ANTHROPIC_API_KEY` depuis
//! l'environnement du processus à chaque appel : ce module est donc la seule
//! source de vérité de cette variable, jamais un état séparé qu'il faudrait
//! resynchroniser.

const API_KEY_VAR: &str = "ANTHROPIC_API_KEY";

/// Récupère la clé API actuellement configurée.
///
/// # Returns
///
/// `Some(clé)` si `ANTHROPIC_API_KEY` est définie et non vide, `None` sinon.
pub fn get_api_key() -> Option<String> {
    std::env::var(API_KEY_VAR).ok().filter(|k| !k.is_empty())
}

/// Configure la clé API en mémoire (variable d'environnement du processus).
///
/// # Arguments
///
/// * `key` - Nouvelle clé API Anthropic.
pub fn set_api_key(key: &str) {
    // SAFETY: écriture rare (appel API explicite de l'utilisateur), jamais
    // sur le chemin chaud des appels LLM (qui ne font que lire la variable).
    unsafe {
        std::env::set_var(API_KEY_VAR, key);
    }
}

/// Supprime la clé API configurée.
pub fn delete_api_key() {
    // SAFETY: voir `set_api_key` — même écriture rare et hors chemin chaud.
    unsafe {
        std::env::remove_var(API_KEY_VAR);
    }
}

/// Indique si une clé API est actuellement configurée.
pub fn has_api_key() -> bool {
    get_api_key().is_some()
}

/// Source de la clé API actuellement configurée.
///
/// # Returns
///
/// Toujours `"env"` sur un serveur headless (pas de trousseau OS).
pub fn key_source() -> &'static str {
    "env"
}
