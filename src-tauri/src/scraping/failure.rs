//! Raisons typées d'échec de la récupération ou de l'extraction d'une offre.
//!
//! Chaque raison est sérialisée en code `snake_case` (ex. `"access_denied"`)
//! dans le champ `message` de `AppError::Scraping` : c'est l'interface qui
//! choisit le texte affiché et l'étape suivante proposée à l'utilisateur, pas
//! le backend. Le détail technique (statut HTTP, erreur réseau) est seulement
//! tracé dans le log de l'application.

/// Raison d'échec de la récupération ou de l'extraction d'une page d'offre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrapingFailure {
    /// URL refusée : schéma autre que http(s), hôte absent, adresse locale ou privée.
    InvalidUrl,
    /// Offre introuvable ou expirée (HTTP 404, 410).
    OfferUnavailable,
    /// Accès refusé par le site (HTTP 401, 403, 429, 999).
    AccessDenied,
    /// Le site n'a pas répondu dans le délai imparti.
    Timeout,
    /// Erreur réseau (DNS, connexion, TLS, statut HTTP inattendu...).
    Network,
    /// La réponse n'est pas une page HTML.
    NotHtml,
    /// La page est vide.
    EmptyPage,
    /// La page dépasse la taille maximale acceptée.
    TooLarge,
    /// La page ne contient pas assez de texte exploitable (coquille JavaScript,
    /// page de challenge, bandeau cookies seul) ou l'extraction n'a trouvé ni
    /// poste ni description.
    InsufficientContent,
}

impl ScrapingFailure {
    /// Code stable de la raison, identique à sa forme sérialisée.
    ///
    /// # Returns
    ///
    /// Le code `snake_case` de la raison (ex. `"access_denied"`).
    pub fn code(self) -> &'static str {
        match self {
            ScrapingFailure::InvalidUrl => "invalid_url",
            ScrapingFailure::OfferUnavailable => "offer_unavailable",
            ScrapingFailure::AccessDenied => "access_denied",
            ScrapingFailure::Timeout => "timeout",
            ScrapingFailure::Network => "network",
            ScrapingFailure::NotHtml => "not_html",
            ScrapingFailure::EmptyPage => "empty_page",
            ScrapingFailure::TooLarge => "too_large",
            ScrapingFailure::InsufficientContent => "insufficient_content",
        }
    }
}

impl std::fmt::Display for ScrapingFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
