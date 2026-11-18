//! Comparaison normalisée d'un intitulé de poste avec un texte : la lettre
//! doit citer l'intitulé exact de l'offre, sans tenir compte de la casse,
//! des espaces ni de la typographie des tirets et des apostrophes.

/// Forme comparable d'un texte : minuscules, tirets et apostrophes
/// typographiques ramenés à leur forme simple, espaces réduits à un seul.
///
/// # Arguments
///
/// * `text` - Texte à normaliser.
///
/// # Returns
///
/// Texte normalisé.
pub fn normalize_for_match(text: &str) -> String {
    let simple: String = text
        .chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{02BC}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            other => other,
        })
        .collect();
    simple.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Vrai si `text` contient l'intitulé `title` une fois les deux normalisés.
/// Un intitulé vide n'impose rien.
///
/// # Arguments
///
/// * `text` - Texte à vérifier (objet de la lettre).
/// * `title` - Intitulé exact du poste.
///
/// # Returns
///
/// Vrai si l'intitulé figure dans le texte.
pub fn contains_title(text: &str, title: &str) -> bool {
    let wanted = normalize_for_match(title);
    wanted.is_empty() || normalize_for_match(text).contains(&wanted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intitule_exact_trouve_malgre_casse_espaces_et_typographie() {
        assert!(contains_title("Candidature : Data  Engineer – Stage", "data engineer - stage"));
        assert!(contains_title("Poste d’ingénieur", "Poste d'ingénieur"));
    }

    #[test]
    fn intitule_reformule_refuse() {
        assert!(!contains_title("Candidature au stage de data engineer", "Data engineer stagiaire"));
    }

    #[test]
    fn intitule_vide_n_impose_rien() {
        assert!(contains_title("Objet", "  "));
    }
}
