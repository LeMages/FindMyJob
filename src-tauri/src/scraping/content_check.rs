//! Détection d'une page sans contenu d'offre exploitable : coquille d'une
//! application JavaScript, page de challenge anti-robot, bandeau cookies
//! seul. Le texte d'une telle page ne doit jamais être envoyé au LLM.

/// Nombre minimal de caractères non blancs d'un texte de page exploitable.
/// Une offre réelle (missions, profil) dépasse largement ce seuil ; un
/// bandeau cookies ou un message « activez JavaScript » reste en dessous.
const MIN_PAGE_TEXT_CHARS: usize = 300;

/// En dessous de cette longueur, la présence d'un marqueur de challenge
/// suffit à rejeter la page. Au-delà, le marqueur peut être un simple
/// élément annexe d'une vraie offre.
const CHALLENGE_MAX_CHARS: usize = 2_000;

/// Formules typiques des pages de challenge et des coquilles JavaScript,
/// comparées en minuscules.
const CHALLENGE_MARKERS: &[&str] = &[
    "just a moment",
    "checking your browser",
    "verify you are human",
    "vérifiez que vous êtes humain",
    "vérification que vous êtes un humain",
    "attention required",
    "enable javascript and cookies",
    "please enable javascript",
    "you need to enable javascript",
    "activez javascript",
    "activer javascript",
    "veuillez activer javascript",
    "captcha",
    "access denied",
    "request unsuccessful",
    "accès refusé",
];

/// Indique si le texte d'une page est insuffisant pour en extraire une offre.
///
/// # Arguments
///
/// * `text` - Texte rendu de la page (après isolement du contenu).
///
/// # Returns
///
/// `true` si le texte est trop court ou si c'est une page de challenge.
pub fn is_insufficient_page_text(text: &str) -> bool {
    let visible = text.chars().filter(|c| !c.is_whitespace()).count();
    if visible < MIN_PAGE_TEXT_CHARS {
        return true;
    }
    if visible >= CHALLENGE_MAX_CHARS {
        return false;
    }
    let lowered = text.to_lowercase();
    CHALLENGE_MARKERS.iter().any(|marker| lowered.contains(marker))
}
