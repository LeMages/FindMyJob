//! Données « clairsemées » d'un rendu de contrôle : valeurs d'exemple
//! dont les champs secondaires (coordonnées, organisations, lieux, niveaux),
//! que le profil peut ne pas renseigner, sont vidés. Une référence correcte
//! n'affiche alors aucun séparateur orphelin (« : | », « | | ») : voir
//! `documents::reference_contract::validate`.

// Importation des modules internes
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
