//! Taille d'un schéma de structured output, telle que la grammaire compilée
//! par l'API la ressent.
//!
//! L'API refuse un schéma trop gros (« The compiled grammar is too large »)
//! sans documenter la limite. On la borne par deux mesures simples, calibrées
//! sur les schémas mesurés (voir `tests/integration/llm_schema_size.rs`) :
//! la taille du JSON et le nombre de nœuds `type`.

// Importation des crates externes
use serde_json::Value;

/// Taille d'un schéma.
///
/// # Attributes
///
/// * `bytes` - Taille du JSON compact.
/// * `type_nodes` - Nombre d'objets portant une clé `type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaSize {
    pub bytes: usize,
    pub type_nodes: usize,
}

/// Taille JSON maximale admise. Mesures de l'audit du 2026-10-20 :
/// `example_data` CV seul (3 032 octets, 67 nœuds) accepté ; enveloppe de
/// templatisation CV (3 225 octets, 70 nœuds) refusée ; la même sans
/// n'importe laquelle de ses propriétés, acceptée.
pub const MAX_SCHEMA_BYTES: usize = 3_100;
/// Nombre maximal de nœuds `type` admis (même calibrage).
pub const MAX_SCHEMA_TYPE_NODES: usize = 68;

/// Profondeur maximale parcourue : un schéma de sortie ne dépasse jamais
/// quelques niveaux, au-delà c'est une erreur de construction.
const MAX_DEPTH: usize = 32;

/// Mesure un schéma.
///
/// # Arguments
///
/// * `schema` - Schéma JSON.
///
/// # Returns
///
/// `SchemaSize`.
pub fn schema_size(schema: &Value) -> SchemaSize {
    SchemaSize { bytes: schema.to_string().len(), type_nodes: count_type_nodes(schema, 0) }
}

impl SchemaSize {
    /// Indique si le schéma reste sous la limite calibrée de l'API.
    ///
    /// # Returns
    ///
    /// Vrai si les deux mesures sont sous leur plafond.
    pub fn fits_api_grammar(&self) -> bool {
        self.bytes <= MAX_SCHEMA_BYTES && self.type_nodes <= MAX_SCHEMA_TYPE_NODES
    }
}

/// Compte les objets qui portent une clé `type`, récursivement.
fn count_type_nodes(value: &Value, depth: usize) -> usize {
    if depth > MAX_DEPTH {
        return 0;
    }
    match value {
        Value::Object(map) => {
            let own = usize::from(map.contains_key("type"));
            own + map.values().map(|v| count_type_nodes(v, depth + 1)).sum::<usize>()
        }
        Value::Array(items) => items.iter().map(|v| count_type_nodes(v, depth + 1)).sum(),
        _ => 0,
    }
}
