//! Types partagés entre toutes les sources de scraping d'offres d'emploi.

// Importation des modules externes
use async_trait::async_trait;

/// Une offre d'emploi normalisée, indépendamment de sa source.
///
/// Attributes:
///     title: Intitulé du poste.
///     company: Nom de l'entreprise.
///     location: Localisation affichée (ville, pays...).
///     url: Lien vers l'offre sur le site source.
///     contract_type: Type de contrat (stage, alternance, CDI...), si connu.
///     posted_at: Date de publication, si connue.
pub struct ScrapedOffer {
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub contract_type: Option<String>,
    pub posted_at: Option<String>,
}

/// Une recherche d'offres à exécuter sur une source.
///
/// Attributes:
///     keywords: Mots-clés de recherche (intitulé de poste, technologies...).
///     location: Ville ou zone recherchée, si l'utilisateur en a précisé une.
///     country_code: Code pays ISO (ex: "FR"), si l'utilisateur en a précisé un.
pub struct SearchQuery {
    pub keywords: String,
    pub location: Option<String>,
    pub country_code: Option<String>,
}

/// Interface commune à toutes les sources de scraping d'offres d'emploi.
///
/// Permet d'ajouter d'autres sources (Indeed, LinkedIn...) plus tard
/// derrière la même interface.
#[async_trait]
pub trait JobScraper {
    /// Exécute les recherches données et retourne les offres trouvées.
    ///
    /// Args:
    ///     queries: Recherches à exécuter sur la source.
    ///
    /// Returns:
    ///     Les offres trouvées, ou une erreur réseau/parsing.
    async fn scrape(
        &self,
        queries: &[SearchQuery],
    ) -> Result<Vec<ScrapedOffer>, Box<dyn std::error::Error>>;
}
