//! Retrait des marqueurs Markdown d'un texte, ligne par ligne : emphase
//! (`**`, `__`, `*`), titres `#`, décorations (`*** Titre ***`), échappements
//! `\`, et conversion des puces `* ` / `• ` en `- `.

/// Caractères pouvant former une décoration de ligne (`*** Titre ***`,
/// `=== Titre ===`) ou une règle horizontale (`---`).
const DECORATION_CHARS: &[char] = &['*', '=', '#', '_', '-', '~'];

/// Longueur minimale d'une suite de caractères pour être une décoration.
const DECORATION_MIN_LEN: usize = 3;

/// Préfixes de puce convertis en `- `.
const BULLET_PREFIXES: &[&str] = &["* ", "• "];

/// Retire les marqueurs Markdown de chaque ligne du texte.
///
/// # Arguments
///
/// * `text` - Texte ligne par ligne (Markdown ou rendu HTML).
///
/// # Returns
///
/// Le texte sans marqueurs Markdown, indentation des lignes conservée.
pub(crate) fn clean_markdown(text: &str) -> String {
    text.lines().map(clean_line).collect::<Vec<_>>().join("\n")
}

/// Nettoie une ligne : échappements, décoration, titre, puce puis emphase.
fn clean_line(line: &str) -> String {
    let body = line.trim_start();
    let indent = &line[..line.len() - body.len()];

    let unescaped = remove_escapes(body.trim_end());
    let undecorated = strip_decoration(&unescaped);
    let without_heading = strip_heading_marker(undecorated);

    match strip_bullet(without_heading) {
        Some(rest) => format!("{indent}- {}", remove_emphasis(rest)),
        None => format!("{indent}{}", remove_emphasis(without_heading)),
    }
}

/// Supprime l'antislash des échappements Markdown (`\-`, `\*`, `\_`...).
fn remove_escapes(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        let escapes_punctuation = c == '\\' && chars.peek().is_some_and(|next| next.is_ascii_punctuation());
        if !escapes_punctuation {
            out.push(c);
        }
    }
    out
}

/// Retire une décoration entourant la ligne (`*** Titre ***`) ; une ligne
/// faite uniquement de décoration (règle horizontale `---`) devient vide.
fn strip_decoration(body: &str) -> &str {
    let leading = body.chars().take_while(|c| DECORATION_CHARS.contains(c)).count();
    if leading == body.chars().count() {
        return if leading >= DECORATION_MIN_LEN { "" } else { body };
    }

    let trailing = body.chars().rev().take_while(|c| DECORATION_CHARS.contains(c)).count();
    if leading < DECORATION_MIN_LEN || trailing < DECORATION_MIN_LEN {
        return body;
    }

    // Les caractères de décoration sont ASCII : 1 caractère = 1 octet.
    body[leading..body.len() - trailing].trim()
}

/// Retire les `#` d'un titre Markdown en début de ligne (`## Missions`).
fn strip_heading_marker(body: &str) -> &str {
    let rest = body.trim_start_matches('#');
    if rest.len() == body.len() {
        return body;
    }
    if rest.is_empty() || rest.starts_with(' ') {
        rest.trim_start()
    } else {
        // `#1 en France` : pas un titre.
        body
    }
}

/// Détecte une puce en début de ligne (`* `, `• `, ou déjà `- `).
///
/// Returns:
///     Le texte après la puce, ou `None` si la ligne n'est pas une puce.
fn strip_bullet(body: &str) -> Option<&str> {
    if let Some(rest) = body.strip_prefix("- ") {
        return Some(rest);
    }
    BULLET_PREFIXES.iter().find_map(|prefix| body.strip_prefix(prefix))
}

/// Supprime les marqueurs d'emphase : `***`, `**`, `__` puis tout `*` restant
/// (italique `*texte*`). Le `*` n'a pas d'usage textuel dans les offres, au
/// contraire du `_` simple (adresses, identifiants), qu'on conserve.
fn remove_emphasis(text: &str) -> String {
    text.replace("__", "").replace('*', "")
}
