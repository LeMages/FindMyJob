//! Repérage des familles importées homonymes d'une police embarquée par
//! Typst ou installée sur le système. Typst charge ces polices avant celles
//! de l'utilisateur : à variante égale, la face chargée en premier gagne, et
//! la version importée ne sert que pour les variantes absentes de l'autre.

// Importation des modules standard
use std::collections::HashSet;

/// Familles fournies par Typst (polices embarquées) et par le système, en
/// minuscules (Typst compare les familles sans tenir compte de la casse).
///
/// Parcourt les polices système : appel coûteux, à faire hors du runtime
/// async et une fois par opération.
///
/// # Returns
///
/// Ensemble des noms de famille en minuscules.
pub fn provided_families() -> HashSet<String> {
    let embedded = typst_kit::fonts::embedded().map(|(_, info)| info.family);
    let system = typst_kit::fonts::system().map(|(_, info)| info.family);
    embedded.chain(system).map(|family| family.to_lowercase()).collect()
}

/// Sélectionne les familles importées déjà fournies par ailleurs.
///
/// # Arguments
///
/// * `imported` - Noms des familles importées.
/// * `provided` - Familles embarquées et système, en minuscules.
///
/// # Returns
///
/// Familles importées homonymes, dans l'ordre de `imported`.
pub fn shadowed_families<'a>(imported: impl IntoIterator<Item = &'a str>, provided: &HashSet<String>) -> Vec<String> {
    imported.into_iter().filter(|family| provided.contains(&family.to_lowercase())).map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homonyme_detecte_sans_tenir_compte_de_la_casse() {
        let provided: HashSet<String> = ["libertinus serif".to_string(), "arial".to_string()].into();
        assert_eq!(shadowed_families(["Arimo", "Libertinus Serif"], &provided), vec!["Libertinus Serif".to_string()]);
    }

    #[test]
    fn polices_embarquees_fournies() {
        let provided = provided_families();
        assert!(provided.contains("libertinus serif"), "{provided:?}");
        assert!(provided.contains("new computer modern"), "{provided:?}");
    }
}
