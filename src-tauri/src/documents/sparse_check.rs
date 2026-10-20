//! Contrôle « clairsemé » d'un template : rendu avec les champs secondaires
//! vides (coordonnées, organisations, lieux, niveaux), que le profil peut
//! ne pas renseigner. Un template correct n'affiche alors aucun séparateur
//! orphelin (« : | », « | | »).
//!
//! Quand des orphelins apparaissent, chaque champ est vidé seul pour
//! désigner celui qui les cause : le feedback dit au LLM quel séparateur
//! conditionner, au lieu d'un constat global qu'il ne sait pas corriger.

// Importation des modules standard
use std::path::PathBuf;

// Importation des modules internes
use crate::documents::typst::{compile_template, CompiledDocument};
use crate::docgen::text_diff::orphan_separators;
use crate::error::AppError;

/// Champs secondaires vidés pour le rendu clairsemé.
pub const SPARSE_FIELDS: [&str; 16] = [
    "title", "email", "telephone", "ville", "zip_code", "linkedin", "github", "portfolio", "entreprise", "lieu",
    "pour_qui", "stack", "url", "ecole", "level", "location",
];

/// Données d'exemple dont les champs nommés sont vidés, partout où ils
/// apparaissent (identité, entrées de listes).
///
/// # Arguments
///
/// * `example` - Données d'exemple sérialisées (telles que rendues).
/// * `fields` - Noms des champs à vider.
///
/// # Returns
///
/// Données sérialisées.
pub fn cleared_data(example: &[u8], fields: &[&str]) -> Result<Vec<u8>, AppError> {
    fn clear(value: &mut serde_json::Value, fields: &[&str]) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, field) in map.iter_mut() {
                    if fields.contains(&key.as_str()) && field.is_string() {
                        *field = serde_json::Value::String(String::new());
                    } else {
                        clear(field, fields);
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(|item| clear(item, fields)),
            _ => {}
        }
    }
    let mut data: serde_json::Value = serde_json::from_slice(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple illisibles : {e}")))?;
    clear(&mut data, fields);
    serde_json::to_vec(&data).map_err(|e| AppError::Validation(format!("Données clairsemées non sérialisables : {e}")))
}

/// Problèmes du rendu clairsemé, un par champ fautif.
///
/// # Arguments
///
/// * `rendered` - Rendu avec tous les champs secondaires vides.
/// * `template` - Source du template.
/// * `example` - Données d'exemple sérialisées.
/// * `root`, `fonts_dir` - Dossiers de compilation.
///
/// # Returns
///
/// Problèmes à renvoyer en feedback (vide si aucun orphelin).
pub async fn sparse_problems(
    rendered: Result<CompiledDocument, String>,
    template: &str,
    example: &[u8],
    root: PathBuf,
    fonts_dir: PathBuf,
) -> Result<Vec<String>, AppError> {
    let rendered = match rendered {
        Ok(rendered) => rendered,
        Err(errors) => {
            return Ok(vec![format!(
                "Le template ne compile pas quand les champs secondaires sont vides ({}) : {errors}",
                SPARSE_FIELDS.join(", ")
            )])
        }
    };
    let orphans = orphan_separators(&rendered.text);
    if orphans.is_empty() {
        return Ok(Vec::new());
    }

    let culprits = orphans_by_field(template, example, root, fonts_dir).await?;
    if culprits.is_empty() {
        // Orphelins qui n'apparaissent qu'en vidant plusieurs champs à la fois.
        return Ok(vec![format!(
            "Rendu avec les champs secondaires vides ({}), le template laisse des séparateurs orphelins : {}. \
             Un champ vide doit faire disparaître sa valeur et le séparateur qui l'accompagne : place le séparateur \
             dans un `if` sur les champs qu'il sépare.",
            SPARSE_FIELDS.join(", "),
            quote(&orphans)
        )]);
    }
    Ok(culprits
        .into_iter()
        .map(|(field, orphans)| {
            format!(
                "Quand le champ `{field}` est vide, le template laisse les séparateurs orphelins {}. Un séparateur \
                 placé avant ou après `{field}` (par exemple « : » entre une langue et son niveau, ou « | » entre \
                 deux valeurs) ne s'affiche que si `{field}` n'est pas vide : place-le dans un `if` qui \
                 teste `{field} != \"\"`.",
                quote(&orphans)
            )
        })
        .collect())
}

/// Vide chaque champ secondaire seul et relève les orphelins qu'il cause.
async fn orphans_by_field(
    template: &str,
    example: &[u8],
    root: PathBuf,
    fonts_dir: PathBuf,
) -> Result<Vec<(&'static str, Vec<String>)>, AppError> {
    let mut data_sets = Vec::with_capacity(SPARSE_FIELDS.len());
    for field in SPARSE_FIELDS {
        data_sets.push((field, cleared_data(example, &[field])?));
    }
    let template = template.to_string();
    tokio::task::spawn_blocking(move || {
        data_sets
            .into_iter()
            .filter_map(|(field, data)| {
                // Le rendu avec tous les champs vides compile : un échec ici ne
                // désigne pas un séparateur orphelin, le champ est ignoré.
                let rendered = compile_template(template.clone(), data, root.clone(), fonts_dir.clone()).ok()?;
                let orphans = orphan_separators(&rendered.text);
                (!orphans.is_empty()).then_some((field, orphans))
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::Validation(format!("Erreur interne de compilation Typst : {e}")))
}

/// Cite des séparateurs orphelins.
fn quote(orphans: &[String]) -> String {
    let mut unique: Vec<&String> = Vec::new();
    for orphan in orphans {
        if !unique.contains(&orphan) {
            unique.push(orphan);
        }
    }
    unique.iter().map(|o| format!("« {o} »")).collect::<Vec<_>>().join(", ")
}
