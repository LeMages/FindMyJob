//! Contrôle du bloc destinataire de la lettre : quand il n'a que
//! l'entreprise et la ville (le cas courant, la rue et le service ne sont
//! pas connus), il doit rester calé contre la marge droite. Un bloc de
//! largeur fixe garde son bord gauche et se retrouve décalé vers le centre.
//! Le contrôle ne s'applique que si le bloc de la référence est lui-même
//! placé à droite.

// Importation des crates externes
use serde_json::{json, Value};

// Importation des modules internes
use crate::documents::typst::CompiledDocument;
use crate::documents::typst_lines::TextLine;
use crate::error::AppError;

/// Marqueurs de l'entreprise et de la ville dans le rendu clairsemé.
const SPARSE_COMPANY: &str = "ZZDESTA";
const SPARSE_LOCATION: &str = "ZZDESTB";

/// Écart toléré (pt) entre le bord droit du bloc et la marge droite (les
/// lignes justifiées ou finissant par une espace dépassent de quelques pt ;
/// un bloc de largeur fixe en est à plus de 100 pt).
const RIGHT_EDGE_TOLERANCE_PT: f64 = 5.0;

/// Données d'exemple du rendu « destinataire clairsemé » : sans lignes
/// supplémentaires, entreprise et ville remplacées par des marqueurs.
///
/// # Arguments
///
/// * `example` - Données d'exemple de la lettre (JSON).
///
/// # Returns
///
/// Données sérialisées, ou erreur si l'exemple n'est pas un objet JSON.
pub fn sparse_recipient_data(example: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut data: Value = serde_json::from_slice(example)
        .map_err(|e| AppError::Validation(format!("Données d'exemple illisibles pour le rendu destinataire : {e}")))?;
    data["recipient"] = json!({
        "company": SPARSE_COMPANY,
        "location": SPARSE_LOCATION,
        "extra_lines": [],
        "contact_name": "",
        "contact_role": "",
    });
    serde_json::to_vec(&data)
        .map_err(|e| AppError::Validation(format!("Données de rendu non sérialisables : {e}")))
}

/// Lignes dont le texte est exactement l'une des valeurs.
fn lines_showing<'a>(lines: &'a [TextLine], values: &[String]) -> Vec<&'a TextLine> {
    lines
        .iter()
        .filter(|line| {
            let text = line.words.join(" ");
            values.iter().any(|v| !v.is_empty() && *v == text)
        })
        .collect()
}

/// Valeurs de texte du destinataire dans les données d'exemple.
fn recipient_values(example: &Value) -> Vec<String> {
    let recipient = &example["recipient"];
    let mut values = Vec::new();
    for key in ["company", "location"] {
        if let Some(text) = recipient[key].as_str() {
            values.push(text.split_whitespace().collect::<Vec<_>>().join(" "));
        }
    }
    for line in recipient["extra_lines"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        values.push(line.split_whitespace().collect::<Vec<_>>().join(" "));
    }
    values
}

/// Bord droit du contenu d'une page : la ligne la plus à droite.
fn page_right_edge(lines: &[TextLine], page: usize) -> f64 {
    lines.iter().filter(|l| l.page == page).map(|l| l.right).fold(f64::MIN, f64::max)
}

/// Vrai si le bloc de la référence (rendu avec l'exemple complet) est placé
/// dans la moitié droite de la page.
fn reference_block_is_on_the_right(rendered: &CompiledDocument, example: &Value) -> bool {
    let block = lines_showing(&rendered.lines, &recipient_values(example));
    let Some(first) = block.first() else {
        return false;
    };
    let left_margin = rendered.lines.iter().filter(|l| l.page == first.page).map(|l| l.left).fold(f64::MAX, f64::min);
    let right_edge = page_right_edge(&rendered.lines, first.page);
    block.iter().map(|l| l.left).fold(f64::MAX, f64::min) > (left_margin + right_edge) / 2.0
}

/// Décalage du bloc destinataire réduit à l'entreprise et la ville, quand
/// celui de l'exemple est dans la moitié droite de la page.
///
/// # Arguments
///
/// * `with_example` - Rendu du template avec les données d'exemple complètes.
/// * `example` - Données d'exemple de la lettre (JSON).
/// * `sparse` - Rendu avec le destinataire clairsemé (`sparse_recipient_data`).
///
/// # Returns
///
/// `Some((bord droit du bloc, marge droite))` en pt si le bloc réduit n'est
/// plus calé contre la marge droite, `None` sinon ou sans objet.
pub fn recipient_misalignment(with_example: &CompiledDocument, example: &Value, sparse: &CompiledDocument) -> Option<(f64, f64)> {
    if !reference_block_is_on_the_right(with_example, example) {
        return None;
    }
    let markers = [SPARSE_COMPANY.to_string(), SPARSE_LOCATION.to_string()];
    let block = lines_showing(&sparse.lines, &markers);
    let first = block.first()?;
    let block_right = block.iter().map(|l| l.right).fold(f64::MIN, f64::max);
    let margin = page_right_edge(&sparse.lines, first.page);
    ((margin - block_right).abs() > RIGHT_EDGE_TOLERANCE_PT).then_some((block_right, margin))
}
