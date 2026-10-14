//! Comptage de mots d'un texte, unité commune des budgets et du linter.

/// Compte les mots d'un texte : suites de caractères séparées par des
/// espaces, qui contiennent au moins une lettre ou un chiffre (une
/// ponctuation isolée, comme « : » en français, n'est pas un mot).
///
/// # Arguments
///
/// * `text` - Texte à mesurer.
///
/// # Returns
///
/// Nombre de mots.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().filter(|w| w.chars().any(char::is_alphanumeric)).count()
}
