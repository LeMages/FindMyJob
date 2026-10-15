//! Normalisation finale des espaces : lignes trimées (indentation des puces
//! conservée), espaces internes réduits, au plus une ligne vide consécutive.

/// Normalise les espaces et sauts de ligne d'un texte.
///
/// # Arguments
///
/// * `text` - Texte ligne par ligne, éventuellement avec `\r\n`.
///
/// # Returns
///
/// Le texte normalisé : 3 sauts de ligne ou plus ramenés à 2, sans lignes
/// vides en début ni en fin.
pub(crate) fn normalize_whitespace(text: &str) -> String {
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = Vec::new();

    for raw in unified.lines() {
        let line = normalize_line(raw);
        let previous_is_blank = lines.last().is_none_or(|last| last.is_empty());
        if line.is_empty() && previous_is_blank {
            continue;
        }
        lines.push(line);
    }

    while lines.last().is_some_and(|last| last.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// Normalise une ligne : espaces internes réduits à un seul, bords trimés.
/// Une puce garde son indentation (en espaces), qui porte l'imbrication.
fn normalize_line(raw: &str) -> String {
    let content = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let is_bullet = content == "-" || content.starts_with("- ");
    if !is_bullet {
        return content;
    }
    if content == "-" {
        // Puce vide (ex. `<li></li>` ou emoji seul) : aucun contenu à garder.
        return String::new();
    }
    let indent_width = raw.chars().take_while(|c| c.is_whitespace()).count();
    format!("{}{content}", " ".repeat(indent_width))
}
