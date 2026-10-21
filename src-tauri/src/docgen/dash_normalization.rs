//! Normalisation déterministe des tirets d'un texte rendu : aucun tiret
//! cadratin (« — »), demi-cadratin (« – »), « --- » ni « -- » ne doit
//! apparaître dans un document ou un message généré.
//!
//! « --- » est traité comme « — » et « -- » comme « – » (Typst ne fait cette
//! conversion qu'en markup, jamais dans une chaîne de `data.json`). Un tiret
//! entouré d'espaces devient « - » dans un champ court et une virgule dans
//! un paragraphe ; entre deux chiffres ou collé à un mot, il devient « - ».
//! Un « - » simple n'est jamais touché, ni les URL, emails et numéros de
//! téléphone.

/// Tiret cadratin.
const EM_DASH: char = '\u{2014}';
/// Tiret demi-cadratin.
const EN_DASH: char = '\u{2013}';
/// Ponctuation devant ou après laquelle une incise ne crée pas de virgule.
const PUNCTUATION: [char; 7] = [',', ';', ':', '.', '!', '?', ')'];
/// Nombre minimal de chiffres d'un numéro de téléphone.
const PHONE_MIN_DIGITS: usize = 8;

/// Nature du texte normalisé, qui décide du sort d'un tiret entre espaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashContext {
    /// Champ court (poste, titre, objet, libellé) : « X — Y » → « X - Y ».
    Short,
    /// Paragraphe ou résumé : une incise entre tirets devient une incise
    /// entre virgules.
    Paragraph,
}

/// Marque interdite relevée dans un texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashMark {
    EmDash,
    EnDash,
    TripleHyphen,
    DoubleHyphen,
}

impl DashMark {
    /// Désignation de la marque, telle qu'écrite dans le texte.
    ///
    /// # Returns
    ///
    /// Libellé lisible (ex. « — » (tiret cadratin)).
    pub fn label(self) -> &'static str {
        match self {
            DashMark::EmDash => "« — » (tiret cadratin)",
            DashMark::EnDash => "« – » (tiret demi-cadratin)",
            DashMark::TripleHyphen => "« --- » (triple trait d'union)",
            DashMark::DoubleHyphen => "« -- » (double trait d'union)",
        }
    }

    /// Substitut à employer à la place de la marque.
    ///
    /// # Returns
    ///
    /// Consigne de remplacement, sans tiret employé comme ponctuation.
    pub fn substitute(self) -> &'static str {
        match self {
            DashMark::EmDash | DashMark::TripleHyphen => {
                "remplace-le par une virgule ou des parenthèses pour une incise, par un deux-points ou un point sinon"
            }
            DashMark::EnDash | DashMark::DoubleHyphen => {
                "remplace-le par un trait d'union simple (-) entre deux nombres, par une virgule ou un deux-points sinon"
            }
        }
    }
}

/// Vrai si un jeton (suite de caractères sans espace) est une URL, un email
/// ou un numéro de téléphone, que la normalisation ne touche jamais.
fn is_protected(token: &str) -> bool {
    let lowered = token.to_lowercase();
    if lowered.contains("://") || lowered.starts_with("www.") {
        return true;
    }
    // Domaine suivi d'un chemin : « github.com/foo--bar ».
    if let Some((host, _)) = token.split_once('/') {
        if host.split('.').filter(|part| !part.is_empty()).count() >= 2 && !host.ends_with('.') {
            return true;
        }
    }
    if let Some((user, domain)) = token.split_once('@') {
        if !user.is_empty() && domain.contains('.') {
            return true;
        }
    }
    is_phone(token)
}

/// Vrai si un jeton ressemble à un numéro de téléphone : assez de chiffres,
/// séparés seulement par des caractères de téléphone.
fn is_phone(token: &str) -> bool {
    let digits = token.chars().filter(char::is_ascii_digit).count();
    digits >= PHONE_MIN_DIGITS
        && !token.contains("--")
        && token.chars().all(|c| c.is_ascii_digit() || "+.-()".contains(c))
}

/// Remplace dans un jeton les suites de traits d'union : « --- » (ou plus)
/// devient « — », « -- » devient « – ».
fn convert_hyphen_runs(token: &str) -> String {
    let mut out = String::with_capacity(token.len());
    let mut run = 0usize;
    for c in token.chars().chain(std::iter::once('\0')) {
        if c == '-' {
            run += 1;
            continue;
        }
        match run {
            0 => {}
            1 => out.push('-'),
            2 => out.push(EN_DASH),
            _ => out.push(EM_DASH),
        }
        run = 0;
        if c != '\0' {
            out.push(c);
        }
    }
    out
}

/// Découpe un texte en caractères marqués « protégés » (URL, email,
/// téléphone), les suites de traits d'union des autres jetons étant déjà
/// converties en tirets.
fn tokenize(text: &str) -> Vec<(char, bool)> {
    let mut chars = Vec::with_capacity(text.len());
    let mut token = String::new();
    for c in text.chars() {
        if c.is_whitespace() {
            flush(&mut token, &mut chars);
            chars.push((c, false));
        } else {
            token.push(c);
        }
    }
    flush(&mut token, &mut chars);
    chars
}

/// Ajoute un jeton terminé aux caractères, puis le vide.
fn flush(token: &mut String, chars: &mut Vec<(char, bool)>) {
    if token.is_empty() {
        return;
    }
    let protected = is_protected(token);
    let converted = if protected { token.clone() } else { convert_hyphen_runs(token) };
    chars.extend(converted.chars().map(|c| (c, protected)));
    token.clear();
}

/// Vrai si le caractère est un tiret interdit hors jeton protégé.
fn is_dash(entry: &(char, bool)) -> bool {
    !entry.1 && (entry.0 == EM_DASH || entry.0 == EN_DASH)
}

/// Retire les espaces en fin de texte produit.
fn trim_end_spaces(out: &mut String) {
    let trimmed = out.trim_end().len();
    out.truncate(trimmed);
}

/// Remplace une incise entourée d'espaces par une virgule, sans doubler une
/// ponctuation déjà présente.
fn push_paragraph_break(out: &mut String, next: char) {
    trim_end_spaces(out);
    let after_punctuation = out.ends_with(|c: char| PUNCTUATION.contains(&c) && c != ')') || out.ends_with('(');
    if PUNCTUATION.contains(&next) || out.ends_with('(') {
        return;
    }
    if after_punctuation {
        out.push(' ');
    } else {
        out.push_str(", ");
    }
}

/// Normalise les tirets d'un texte.
///
/// # Arguments
///
/// * `text` - Texte à normaliser.
/// * `context` - Champ court ou paragraphe.
///
/// # Returns
///
/// Texte sans « — », « – », « --- » ni « -- » (hors URL, email, téléphone).
pub fn normalize_dashes(text: &str, context: DashContext) -> String {
    let chars = tokenize(text);
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if !is_dash(&chars[i]) {
            out.push(chars[i].0);
            i += 1;
            continue;
        }
        // Une suite de tirets collés compte pour un seul.
        let mut end = i;
        while end < chars.len() && is_dash(&chars[end]) {
            end += 1;
        }
        let space_before = i > 0 && chars[i - 1].0.is_whitespace();
        let space_after = end < chars.len() && chars[end].0.is_whitespace();
        let prev = chars[..i].iter().rev().map(|c| c.0).find(|c| !c.is_whitespace());
        let next_index = (end..chars.len()).find(|&k| !chars[k].0.is_whitespace());
        let next = next_index.map(|k| chars[k].0);
        i = end;

        // ------------------------------------------------------------------
        // 1. Tiret en bord de texte suivi ou précédé d'un espace (puce) :
        //    retiré avec ses espaces
        // ------------------------------------------------------------------
        if (prev.is_none() && space_after) || (next.is_none() && space_before) {
            trim_end_spaces(&mut out);
            i = next_index.unwrap_or(chars.len());
            continue;
        }
        // ------------------------------------------------------------------
        // 2. Entre deux chiffres ou collé : trait d'union, espaces conservés
        // ------------------------------------------------------------------
        let numeric = prev.is_some_and(|c| c.is_ascii_digit()) && next.is_some_and(|c| c.is_ascii_digit());
        if numeric || (!space_before && !space_after) {
            out.push('-');
            continue;
        }
        // ------------------------------------------------------------------
        // 3. Entouré d'espaces : « - » en champ court, virgule en paragraphe
        // ------------------------------------------------------------------
        match context {
            DashContext::Short => {
                trim_end_spaces(&mut out);
                out.push_str(" - ");
            }
            DashContext::Paragraph => push_paragraph_break(&mut out, next.unwrap_or(' ')),
        }
        i = next_index.unwrap_or(chars.len());
    }
    out
}

/// Relève les marques interdites d'un texte, hors URL, email et téléphone.
///
/// # Arguments
///
/// * `text` - Texte à contrôler.
///
/// # Returns
///
/// Marques présentes, chacune une fois, dans un ordre fixe.
pub fn dash_marks(text: &str) -> Vec<DashMark> {
    let tokens: Vec<&str> = text.split_whitespace().filter(|t| !is_protected(t)).collect();
    let has = |pattern: &dyn Fn(&str) -> bool| tokens.iter().any(|t| pattern(t));
    let mut marks = Vec::new();
    if has(&|t| t.contains(EM_DASH)) {
        marks.push(DashMark::EmDash);
    }
    if has(&|t| t.contains(EN_DASH)) {
        marks.push(DashMark::EnDash);
    }
    if has(&|t| t.contains("---")) {
        marks.push(DashMark::TripleHyphen);
    }
    // Un « -- » qui n'est qu'une partie d'un « --- » est déjà relevé.
    if has(&|t| t.split("---").any(|part| part.contains("--"))) {
        marks.push(DashMark::DoubleHyphen);
    }
    marks
}

/// Normalise un message de candidature : la ligne d'objet est un champ
/// court, les autres lignes des paragraphes.
///
/// # Arguments
///
/// * `text` - Message rédigé.
///
/// # Returns
///
/// Message sans « — », « – », « --- » ni « -- ».
pub fn normalize_application_message(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let lowered = line.trim_start().to_lowercase();
            let is_subject = lowered.starts_with("objet") || lowered.starts_with("subject");
            normalize_dashes(line, if is_subject { DashContext::Short } else { DashContext::Paragraph })
        })
        .collect::<Vec<_>>()
        .join("\n")
}
