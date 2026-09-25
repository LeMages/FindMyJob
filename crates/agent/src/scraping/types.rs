//! Types partagés entre toutes les sources de scraping d'offres d'emploi.

// Importation des crates externes
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Une offre d'emploi normalisée, indépendamment de sa source.
///
/// Attributes:
///     source: Identifiant de la source ("wttj", "indeed"...).
///     source_id: Identifiant de l'offre côté source (slug, ID...), si connu.
///     title: Intitulé du poste.
///     company: Nom de l'entreprise.
///     location: Localisation affichée (ville, pays...).
///     url: Lien vers l'offre sur le site source.
///     contract_type: Type de contrat (stage, alternance, CDI...), si connu.
///     posted_at: Date de publication, si connue.
///     scraped_at: Date à laquelle l'offre a été récupérée.
///     description: Texte brut de l'offre, assemblé en sections, si connu.
///     description_raw: Champs d'origine ayant servi à construire `description`.
pub struct ScrapedOffer {
    pub source: String,
    pub source_id: Option<String>,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub contract_type: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub scraped_at: DateTime<Utc>,
    pub description: Option<String>,
    pub description_raw: Option<DescriptionRaw>,
}

/// Provenance des champs d'une description d'offre.
#[derive(Serialize)]
pub enum DescriptionSource {
    /// Champ `profile` de l'index Algolia (repli, profil recherché seul).
    #[serde(rename = "algolia")]
    Algolia,
}

/// Champs HTML/Markdown d'origine d'une description, stockés tels quels
/// (colonne `offers.description_raw`) pour rejouer la conversion sans réseau.
///
/// Attributes:
///     description: Descriptif du poste, si fourni par la source.
///     profile: Profil recherché, si fourni par la source.
///     recruitment_process: Déroulement des entretiens, si fourni par la source.
///     source: Provenance des champs.
#[derive(Serialize)]
pub struct DescriptionRaw {
    pub description: Option<String>,
    pub profile: Option<String>,
    pub recruitment_process: Option<String>,
    pub source: DescriptionSource,
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
