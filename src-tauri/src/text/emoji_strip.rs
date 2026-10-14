//! Suppression des emojis par plages Unicode ; un emoji en tête de ligne,
//! utilisé comme puce, devient `- `.

/// Plages Unicode considérées comme des emojis (bornes incluses).
const EMOJI_RANGES: &[(u32, u32)] = &[
    // Pictogrammes, symboles, émoticônes, drapeaux (1F1E6–1F1FF inclus).
    (0x1F000, 0x1FAFF),
    // Symboles divers et dingbats (☀, ✅, ✨, ❤...).
    (0x2600, 0x27BF),
    // Flèches et symboles divers (⭐, ⬆...).
    (0x2B00, 0x2BFF),
    // Sélecteur de variation emoji.
    (0xFE0F, 0xFE0F),
    // Zero-width joiner (séquences composées).
    (0x200D, 0x200D),
    // Caractères de tag (drapeaux régionaux type Écosse).
    (0xE0020, 0xE007F),
];

/// Supprime les emojis de chaque ligne du texte.
///
/// # Arguments
///
/// * `text` - Texte ligne par ligne.
///
/// # Returns
///
/// Le texte sans emojis ; une ligne commençant par un emoji est préfixée
/// par `- ` (sauf si elle est déjà une puce ou devient vide).
pub(crate) fn strip_emojis(text: &str) -> String {
    text.lines().map(strip_line).collect::<Vec<_>>().join("\n")
}

/// Indique si un caractère appartient à une plage emoji.
fn is_emoji(c: char) -> bool {
    let code = c as u32;
    EMOJI_RANGES.iter().any(|&(start, end)| (start..=end).contains(&code))
}

/// Supprime les emojis d'une ligne, en convertissant un emoji de tête en puce.
fn strip_line(line: &str) -> String {
    let body = line.trim_start();
    let indent = &line[..line.len() - body.len()];
    let starts_with_emoji = body.chars().next().is_some_and(is_emoji);
    let cleaned: String = body.chars().filter(|c| !is_emoji(*c)).collect();

    if !starts_with_emoji {
        return format!("{indent}{cleaned}");
    }

    let rest = cleaned.trim_start();
    if rest.is_empty() || rest.starts_with("- ") {
        format!("{indent}{rest}")
    } else {
        format!("{indent}- {rest}")
    }
}
