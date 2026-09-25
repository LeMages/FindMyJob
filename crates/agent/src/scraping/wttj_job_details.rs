//! Enrichissement des offres WTTJ via l'API publique v1 (détail d'une
//! offre) : l'index Algolia ne fournit que le profil recherché, l'API v1
//! renvoie le texte complet (descriptif, profil, déroulement des entretiens).

// Importation des modules standard
use std::time::Duration;

// Importation des crates externes
use reqwest::Client;
use serde::Deserialize;

// Importation des modules internes
use super::types::ScrapedOffer;
use findmyjob_core::text::assemble_offer_description;

const JOB_DETAILS_BASE_URL: &str = "https://api.welcometothejungle.com/api/v1/organizations";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Pause entre deux appels : API publique non documentée, on reste poli.
const CALL_DELAY: Duration = Duration::from_millis(400);

/// Réponse de `GET /api/v1/organizations/{org}/jobs/{slug}`, réduite aux
/// champs utiles (la présentation de l'entreprise est volontairement exclue).
#[derive(Deserialize)]
struct JobDetailsResponse {
    job: JobDetails,
}

/// Champs HTML du détail d'une offre ; chacun peut être absent, `null`
/// ou `"<p>null</p>"` selon l'ATS.
#[derive(Deserialize)]
struct JobDetails {
    description: Option<String>,
    profile: Option<String>,
    recruitment_process: Option<String>,
}

/// Récupère le détail d'une offre sur l'API v1.
///
/// # Arguments
///
/// * `client` - Client HTTP partagé.
/// * `organization_slug` - Slug de l'entreprise (`organization.slug` Algolia).
/// * `job_slug` - Slug de l'offre (`slug` Algolia, `source_id` en base).
///
/// # Returns
///
/// Le détail de l'offre, ou une erreur réseau, de statut HTTP, de timeout
/// ou de JSON inattendu.
async fn fetch_job_details(
    client: &Client,
    organization_slug: &str,
    job_slug: &str,
) -> Result<JobDetails, reqwest::Error> {
    let url = format!("{JOB_DETAILS_BASE_URL}/{organization_slug}/jobs/{job_slug}");
    let response: JobDetailsResponse = client
        .get(url)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response.job)
}

/// Remplace la description (repli Algolia) d'une offre par le texte complet
/// issu de l'API v1.
///
/// # Arguments
///
/// * `offer` - Offre à mettre à jour.
/// * `details` - Détail récupéré sur l'API v1.
///
/// # Returns
///
/// `true` si l'offre a été mise à jour, `false` si toutes les sections sont
/// vides après conversion (l'offre garde alors son repli Algolia).
fn apply_job_details(offer: &mut ScrapedOffer, details: JobDetails) -> bool {
    let Some(description) = assemble_offer_description(
        details.description.as_deref(),
        details.profile.as_deref(),
        details.recruitment_process.as_deref(),
    ) else {
        return false;
    };

    offer.description = Some(description);
    true
}

/// Enrichit une offre, en journalisant la cause si elle garde son repli.
///
/// # Returns
///
/// `true` si l'offre a reçu sa description complète.
async fn enrich_offer(client: &Client, offer: &mut ScrapedOffer) -> bool {
    let (Some(organization_slug), Some(job_slug)) = (offer.organization_slug.clone(), offer.source_id.clone()) else {
        eprintln!("[scraping::wttj] avertissement : slugs manquants pour {}, repli Algolia", offer.url);
        return false;
    };

    match fetch_job_details(client, &organization_slug, &job_slug).await {
        Ok(details) => {
            let applied = apply_job_details(offer, details);
            if !applied {
                eprintln!("[scraping::wttj] avertissement : API v1 sans contenu pour {}, repli Algolia", offer.url);
            }
            applied
        }
        Err(e) => {
            eprintln!("[scraping::wttj] avertissement : API v1 en échec pour {} ({e}), repli Algolia", offer.url);
            false
        }
    }
}

/// Enrichit chaque offre avec sa description complète (API v1). Un échec
/// n'interrompt jamais le batch : l'offre garde sa description Algolia.
///
/// # Arguments
///
/// * `client` - Client HTTP partagé.
/// * `offers` - Offres à enrichir (nouvelles offres uniquement, après dédup).
///
/// # Returns
///
/// Nombre d'offres ayant reçu leur description complète.
pub async fn enrich_offers(client: &Client, offers: &mut [ScrapedOffer]) -> usize {
    let mut enriched = 0;

    for (i, offer) in offers.iter_mut().enumerate() {
        if i > 0 {
            tokio::time::sleep(CALL_DELAY).await;
        }
        if enrich_offer(client, offer).await {
            enriched += 1;
        }
    }

    enriched
}
