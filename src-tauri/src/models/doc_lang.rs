//! Langues de document prises en charge et leurs noms : la seule source
//! de ces valeurs (templatisation, analyse d'offre, rédaction, traductions,
//! langue du profil). Les données gardent le code de langue (`"fr"`,
//! `"en"`), validé à chaque entrée dans l'application.

/// Codes des langues de document prises en charge.
pub const SUPPORTED_LANGS: [&str; 2] = ["fr", "en"];

/// Langue du profil par défaut, quand l'utilisateur n'en a pas choisi.
pub const DEFAULT_PROFILE_LANG: &str = "fr";

/// Indique si un code de langue est pris en charge.
///
/// # Arguments
///
/// * `code` - Code de langue.
///
/// # Returns
///
/// Vrai pour `"fr"` et `"en"`.
pub fn is_supported(code: &str) -> bool {
    SUPPORTED_LANGS.contains(&code)
}

/// Nom d'une langue, en français, pour les prompts et les messages.
///
/// # Arguments
///
/// * `code` - Code d'une langue prise en charge (validé en amont).
///
/// # Returns
///
/// « français » ou « anglais ».
pub fn language_name(code: &str) -> &'static str {
    debug_assert!(is_supported(code), "langue non prise en charge : {code}");
    match code {
        "en" => "anglais",
        "fr" => "français",
        _ => "langue inconnue",
    }
}

/// Sens d'une traduction, en toutes lettres (« du français vers l'anglais »).
///
/// # Arguments
///
/// * `from` - Langue source.
/// * `to` - Langue cible.
///
/// # Returns
///
/// Le sens de la traduction, pour un prompt.
pub fn direction(from: &str, to: &str) -> String {
    let source = match from {
        "en" => format!("de l'{}", language_name(from)),
        _ => format!("du {}", language_name(from)),
    };
    let destination = match to {
        "en" => format!("l'{}", language_name(to)),
        _ => format!("le {}", language_name(to)),
    };
    format!("{source} vers {destination}")
}
