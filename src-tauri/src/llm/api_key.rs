//! Clé API Anthropic, transmise au client HTTP par l'appelant.
//!
//! `core` ne sait pas d'où vient la clé (fichier `.env` ou trousseau du
//! système) : il la reçoit déjà résolue. Le type empêche toute fuite
//! accidentelle — ni `Display` ni `Serialize`, et un `Debug` masqué — pour
//! qu'elle n'apparaisse jamais dans une trace ou un message d'erreur.

/// Nombre de caractères de fin conservés pour identifier une clé.
const VISIBLE_SUFFIX_LEN: usize = 4;

/// Clé API Anthropic non vide, sans espaces autour.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Construit une clé depuis une saisie ou une variable d'environnement.
    ///
    /// # Arguments
    ///
    /// * `raw` - Valeur brute ; les espaces autour sont retirés.
    ///
    /// # Returns
    ///
    /// La clé, ou `None` si la valeur est vide une fois nettoyée.
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| Self(trimmed.to_string()))
    }

    /// Derniers caractères de la clé, pour l'identifier sans la révéler.
    ///
    /// # Returns
    ///
    /// Les 4 derniers caractères (la clé entière si elle est plus courte).
    pub fn last4(&self) -> String {
        let count = self.0.chars().count();
        self.0.chars().skip(count.saturating_sub(VISIBLE_SUFFIX_LEN)).collect()
    }

    /// Valeur complète, destinée au trousseau du système ou au header
    /// d'authentification — jamais à une trace ni au frontend.
    ///
    /// # Returns
    ///
    /// La clé en clair.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(<masquée>)")
    }
}
