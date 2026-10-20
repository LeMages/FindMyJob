//! Rattachement des zones balisées aux champs de prose des données
//! d'exemple, et construction des consignes (`ProseGuidance`).
//!
//! Une zone est retrouvée par son texte : la plus longue suite de mots
//! commune avec un champ de prose. Les mots qui dépassent au bord (un
//! libellé comme « Objet : » écrit sur la même ligne) sont tolérés, dans la
//! limite de `MAX_EDGE_WORDS`, si la partie commune fait plus de la moitié
//! de la zone. La partie d'un champ qui n'est couverte par
//! aucune balise devient une zone `[VARIABLE]` implicite, de sorte que les
//! zones d'un champ balisé le recomposent toujours exactement.
//!
//! Une zone `[FIXE]` posée sur un texte qui n'est pas de la prose (formule
//! de politesse, texte en dur) est sans effet : ce texte est déjà reproduit
//! tel quel. Une zone `[VARIABLE]` ou `[ADAPTABLE]` hors prose est une
//! erreur de balisage.

// Importation des modules internes
use crate::docgen::example_facts::word_keys;
use crate::docgen::text_diff::{normalize_quotes, words};
use crate::documents::markup::model::{MarkupError, MarkupZone};
use crate::models::{ProseField, ProseGuidance, ProseZone, ZoneMode};

/// Mots tolérés de chaque côté d'une zone hors du champ qui la contient.
const MAX_EDGE_WORDS: usize = 4;

/// Zone retrouvée dans un champ : indices de mots `[start, end)`.
struct Placement<'a> {
    field: usize,
    start: usize,
    end: usize,
    zone: &'a MarkupZone,
}

/// Rattache les zones aux champs de prose et construit les consignes.
///
/// # Arguments
///
/// * `general` - Consignes générales lues dans la référence.
/// * `zones` - Zones balisées, dans l'ordre du source.
/// * `fields` - Champs de prose des données d'exemple (chemins de `guidance`).
///
/// # Returns
///
/// Les consignes, ou les erreurs de balisage (ligne de chaque balise fautive).
pub fn attach_zones(
    general: &str,
    zones: &[MarkupZone],
    fields: &[ProseField],
) -> Result<ProseGuidance, Vec<MarkupError>> {
    // ------------------------------------------------------------------
    // 1. Chaque zone est placée dans le champ qui la contient
    // ------------------------------------------------------------------
    let field_keys: Vec<Vec<String>> = fields.iter().map(|f| word_keys(&f.text)).collect();
    let mut placements = Vec::new();
    let mut errors = Vec::new();
    for zone in zones {
        match place(zone, &field_keys) {
            Ok(Some(placement)) => placements.push(placement),
            Ok(None) => {}
            Err(message) => errors.push(MarkupError { line: zone.line, message }),
        }
    }

    // ------------------------------------------------------------------
    // 2. Par champ : zones dans l'ordre, sans chevauchement, trous comblés
    //    par des zones [VARIABLE] implicites
    // ------------------------------------------------------------------
    let mut prose_zones = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let mut own: Vec<&Placement> = placements.iter().filter(|p| p.field == index).collect();
        own.sort_by_key(|p| p.start);
        match field_zones(field, &own) {
            Ok(zones) => prose_zones.extend(zones),
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        errors.sort_by_key(|e| e.line);
        return Err(errors);
    }
    Ok(ProseGuidance { general: general.to_string(), zones: prose_zones })
}

/// Place une zone dans le champ de prose qui la contient.
///
/// # Returns
///
/// `Some(placement)`, `None` pour une zone [FIXE] hors prose (sans effet),
/// ou le message d'erreur.
fn place<'a>(zone: &'a MarkupZone, field_keys: &[Vec<String>]) -> Result<Option<Placement<'a>>, String> {
    let zone_keys = word_keys(&zone.text);
    let best = field_keys
        .iter()
        .enumerate()
        .map(|(field, keys)| (field, longest_common_run(&zone_keys, keys)))
        .max_by_key(|(_, run)| run.length);
    // La partie commune doit être l'essentiel de la zone : un mot isolé en
    // commun (« avec ») ne rattache pas une formule à un paragraphe.
    let fits = |run: &CommonRun| {
        run.length * 2 > zone_keys.len()
            && run.zone_start <= MAX_EDGE_WORDS
            && zone_keys.len() - (run.zone_start + run.length) <= MAX_EDGE_WORDS
    };
    match best {
        Some((field, run)) if fits(&run) => {
            Ok(Some(Placement { field, start: run.field_start, end: run.field_start + run.length, zone }))
        }
        _ if zone.mode == ZoneMode::Fixe => Ok(None),
        _ => Err(format!(
            "la zone {} « {} » ne correspond à aucun texte rédigé (objet, paragraphes de la lettre ; titre, \
             résumé du CV). Seule une zone [FIXE] peut porter sur un autre texte.",
            zone.mode.tag(),
            excerpt(&zone.text)
        )),
    }
}

/// Zones d'un champ, trous compris.
fn field_zones(field: &ProseField, placements: &[&Placement]) -> Result<Vec<ProseZone>, MarkupError> {
    if placements.is_empty() {
        return Ok(Vec::new());
    }
    let normalized = normalize_quotes(&field.text);
    let tokens = words(&normalized);
    let text_of = |start: usize, end: usize| tokens[start..end].join(" ");
    let implicit = |start: usize, end: usize| ProseZone {
        field: field.path.clone(),
        mode: ZoneMode::Variable,
        instruction: String::new(),
        text: text_of(start, end),
    };
    let mut zones: Vec<ProseZone> = Vec::new();
    let mut cursor = 0;
    for placement in placements {
        if placement.start < cursor {
            return Err(MarkupError {
                line: placement.zone.line,
                message: format!("cette zone recouvre la zone précédente du même texte ({}).", field.path),
            });
        }
        // Une ponctuation isolée entre deux zones (« – ») reste dans la zone
        // suivante : les zones recomposent ainsi exactement le champ.
        let start = if has_letters(&tokens[cursor..placement.start]) {
            zones.push(implicit(cursor, placement.start));
            placement.start
        } else {
            cursor
        };
        zones.push(ProseZone {
            field: field.path.clone(),
            mode: placement.zone.mode,
            instruction: placement.zone.instruction.clone(),
            text: text_of(start, placement.end),
        });
        cursor = placement.end;
    }
    if has_letters(&tokens[cursor..]) {
        zones.push(implicit(cursor, tokens.len()));
    } else if let Some(last) = zones.last_mut() {
        for token in &tokens[cursor..] {
            last.text.push(' ');
            last.text.push_str(token);
        }
    }
    Ok(zones)
}

/// Plus longue suite de mots commune entre une zone et un champ.
#[derive(Debug, Clone, Copy, Default)]
struct CommonRun {
    zone_start: usize,
    field_start: usize,
    length: usize,
}

/// Plus longue sous-suite contiguë commune (programmation dynamique sur
/// une ligne, O(n·m) en temps, O(m) en mémoire).
fn longest_common_run(zone: &[String], field: &[String]) -> CommonRun {
    let mut best = CommonRun::default();
    let mut previous = vec![0usize; field.len() + 1];
    for (i, zone_word) in zone.iter().enumerate() {
        let mut current = vec![0usize; field.len() + 1];
        for (j, field_word) in field.iter().enumerate() {
            if zone_word == field_word {
                current[j + 1] = previous[j] + 1;
                if current[j + 1] > best.length {
                    let length = current[j + 1];
                    best = CommonRun { zone_start: i + 1 - length, field_start: j + 1 - length, length };
                }
            }
        }
        previous = current;
    }
    best
}

/// Vrai si l'un des mots contient une lettre ou un chiffre.
fn has_letters(tokens: &[&str]) -> bool {
    tokens.iter().any(|t| t.chars().any(char::is_alphanumeric))
}

/// Début d'un texte, pour un message d'erreur.
fn excerpt(text: &str) -> String {
    const MAX_CHARS: usize = 60;
    if text.chars().count() <= MAX_CHARS {
        return text.to_string();
    }
    format!("{}…", text.chars().take(MAX_CHARS).collect::<String>())
}
