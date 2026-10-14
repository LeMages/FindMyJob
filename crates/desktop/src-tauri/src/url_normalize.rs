//! Normalisation des URL d'offre pour la détection de doublons : deux liens
//! vers la même offre (avec ou sans `www.`, paramètres de suivi, fragment,
//! slash final, casse) doivent être reconnus comme identiques.

/// Normalise une URL d'offre pour comparaison.
///
/// Retire le schéma `http(s)://`, le préfixe `www.`, le fragment (`#…`), la
/// query (`?…`) et les slashs finaux, puis passe en minuscules.
///
/// # Arguments
///
/// * `url` - URL brute (saisie ou stockée).
///
/// # Returns
///
/// La forme normalisée, ou `None` si l'URL est vide une fois normalisée.
pub fn normalize_offer_url(url: &str) -> Option<String> {
    let lowered = url.trim().to_lowercase();
    let without_scheme = lowered
        .strip_prefix("https://")
        .or_else(|| lowered.strip_prefix("http://"))
        .unwrap_or(&lowered);
    let without_www = without_scheme.strip_prefix("www.").unwrap_or(without_scheme);
    let without_fragment = without_www.split('#').next().unwrap_or("");
    let without_query = without_fragment.split('?').next().unwrap_or("");
    let normalized = without_query.trim_end_matches('/');
    (!normalized.is_empty()).then(|| normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_offer_url;

    #[test]
    fn retire_schema_www_query_fragment_et_slash_final() {
        assert_eq!(
            normalize_offer_url("  HTTPS://www.Exemple.fr/Offres/42/?utm_source=x#postuler "),
            Some("exemple.fr/offres/42".to_string())
        );
    }

    #[test]
    fn variantes_d_une_meme_offre_sont_egales() {
        let a = normalize_offer_url("http://exemple.fr/offres/42");
        let b = normalize_offer_url("https://www.exemple.fr/offres/42/");
        let c = normalize_offer_url("exemple.fr/offres/42?ref=newsletter");
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn offres_differentes_restent_distinctes() {
        assert_ne!(
            normalize_offer_url("https://exemple.fr/offres/42"),
            normalize_offer_url("https://exemple.fr/offres/43")
        );
    }

    #[test]
    fn url_vide_donne_none() {
        assert_eq!(normalize_offer_url("   "), None);
        assert_eq!(normalize_offer_url("https://"), None);
        assert_eq!(normalize_offer_url("https://www./"), None);
    }

    #[test]
    fn schema_au_milieu_n_est_pas_retire() {
        assert_eq!(
            normalize_offer_url("exemple.fr/redirect/https://autre.fr"),
            Some("exemple.fr/redirect/https://autre.fr".to_string())
        );
    }
}
