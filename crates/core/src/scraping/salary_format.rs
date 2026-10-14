//! Mise en forme lisible du `baseSalary` JSON-LD d'une offre
//! (`MonetaryAmount` / `QuantitativeValue` de schema.org).

// Importation des crates externes
use serde_json::Value;

/// Formate un `baseSalary` en texte lisible (« 35 000 – 40 000 € par an »).
///
/// Accepte un nombre, une chaîne, ou un `MonetaryAmount` dont `value` est un
/// nombre, une chaîne ou un `QuantitativeValue` (`value`, `minValue`,
/// `maxValue`, `unitText`). Les nombres en chaîne au format français
/// (« 1 200,50 ») sont reconnus.
///
/// # Arguments
///
/// * `base_salary` - Valeur JSON-LD `baseSalary`.
///
/// # Returns
///
/// Le texte, ou `None` si le salaire est absent, vide ou nul.
pub fn format_base_salary(base_salary: &Value) -> Option<String> {
    match base_salary {
        Value::Number(_) | Value::String(_) => format_amount(base_salary, None, None),
        Value::Object(map) => {
            let currency = map.get("currency").and_then(Value::as_str);
            match map.get("value") {
                Some(Value::Object(quantity)) => {
                    let unit = quantity.get("unitText").and_then(Value::as_str);
                    format_quantity(quantity, currency, unit)
                }
                Some(value) => format_amount(value, currency, map.get("unitText").and_then(Value::as_str)),
                None => None,
            }
        }
        Value::Array(items) => items.iter().find_map(format_base_salary),
        _ => None,
    }
}

/// Formate un `QuantitativeValue` : valeur exacte, ou fourchette min–max.
fn format_quantity(
    quantity: &serde_json::Map<String, Value>,
    currency: Option<&str>,
    unit: Option<&str>,
) -> Option<String> {
    if let Some(value) = quantity.get("value") {
        if let Some(text) = format_amount(value, currency, unit) {
            return Some(text);
        }
    }
    let min = quantity.get("minValue").and_then(parse_amount);
    let max = quantity.get("maxValue").and_then(parse_amount);
    let range = match (min, max) {
        (Some(min), Some(max)) if (min - max).abs() < f64::EPSILON => format_number(min),
        (Some(min), Some(max)) => format!("{} – {}", format_number(min), format_number(max)),
        (Some(min), None) => format!("À partir de {}", format_number(min)),
        (None, Some(max)) => format!("Jusqu'à {}", format_number(max)),
        (None, None) => return None,
    };
    Some(with_currency_and_unit(range, currency, unit))
}

/// Formate un montant simple (nombre ou chaîne numérique). Une chaîne non
/// numérique (« Selon profil ») est gardée telle quelle.
fn format_amount(value: &Value, currency: Option<&str>, unit: Option<&str>) -> Option<String> {
    if let Some(amount) = parse_amount(value) {
        return Some(with_currency_and_unit(format_number(amount), currency, unit));
    }
    let text = value.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Lit un montant positif depuis un nombre JSON ou une chaîne au format
/// anglais (« 1200.50 ») ou français (« 1 200,50 »). Zéro vaut absence :
/// certains sites l'utilisent pour « non communiqué ».
fn parse_amount(value: &Value) -> Option<f64> {
    let amount = match value {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) => parse_amount_text(s)?,
        _ => return None,
    };
    (amount > 0.0 && amount.is_finite()).then_some(amount)
}

/// Parse un nombre écrit en texte, séparateurs français ou anglais.
fn parse_amount_text(raw: &str) -> Option<f64> {
    let compact: String = raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .collect();
    let normalized = match (compact.contains(','), compact.contains('.')) {
        // « 1.200,50 » : point = milliers, virgule = décimales.
        (true, true) => compact.replace('.', "").replace(',', "."),
        // « 1200,50 » : virgule décimale.
        (true, false) => compact.replace(',', "."),
        _ => compact,
    };
    normalized.parse::<f64>().ok()
}

/// Écrit un nombre à la française : milliers séparés par une espace,
/// décimales (2 chiffres) après une virgule seulement si nécessaires.
fn format_number(amount: f64) -> String {
    let cents = (amount * 100.0).round() as u64;
    let (integer, decimals) = (cents / 100, cents % 100);
    let digits = integer.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    if decimals == 0 {
        grouped
    } else {
        format!("{grouped},{decimals:02}")
    }
}

/// Ajoute la devise (symbole si connu) et la période au montant.
fn with_currency_and_unit(amount: String, currency: Option<&str>, unit: Option<&str>) -> String {
    let mut text = amount;
    if let Some(currency) = currency.map(str::trim).filter(|c| !c.is_empty()) {
        let symbol = match currency.to_ascii_uppercase().as_str() {
            "EUR" => "€".to_string(),
            "USD" => "$".to_string(),
            "GBP" => "£".to_string(),
            other => other.to_string(),
        };
        text = format!("{text} {symbol}");
    }
    if let Some(period) = unit.and_then(period_label) {
        text = format!("{text} {period}");
    }
    text
}

/// Libellé français d'une période `unitText` de schema.org.
fn period_label(unit: &str) -> Option<&'static str> {
    match unit.trim().to_ascii_uppercase().as_str() {
        "HOUR" => Some("de l'heure"),
        "DAY" => Some("par jour"),
        "WEEK" => Some("par semaine"),
        "MONTH" => Some("par mois"),
        "YEAR" => Some("par an"),
        _ => None,
    }
}
