//! Découpage d'une description du profil (expérience, projet, formation)
//! en intro et éléments de liste.
//!
//! Règle de saisie : une ligne qui commence par `-` est un élément de liste,
//! sauf si le `-` est suivi d'un chiffre (`-30 %`, `-5°C` restent du texte).
//! Les lignes avant le premier élément forment l'intro ; une ligne sans
//! tiret qui suit un élément en est la suite. Seul `-` crée un élément :
//! `•`, `*`, `–` et `—` en tête de ligne sont du texte.
//!
//! Le frontend a un miroir de ce parseur (`src/composables/parseDescriptionList.ts`)
//! pour l'affichage ; les deux sont testés sur la même table de cas
//! (`tests/fixtures/description_list_cases.json`).

/// Description découpée : lignes d'intro, puis éléments (chaque élément a
/// sa première ligne et ses éventuelles lignes de suite). Aucune ligne n'est
/// vide et aucune ne porte de symbole de liste.
///
/// # Attributes
///
/// * `intro` - Lignes avant le premier élément, sans symbole.
/// * `items` - Éléments de liste, chacun formé d'au moins une ligne.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DescriptionList {
    pub intro: Vec<String>,
    pub items: Vec<Vec<String>>,
}

/// Contenu d'un élément si la ligne (déjà trimée) en ouvre un, `None` si
/// c'est du texte. Un contenu vide (`-` seul) donne `Some("")`.
fn item_content(line: &str) -> Option<&str> {
    let rest = line.strip_prefix('-')?;
    // Un `-` collé à un chiffre est un signe (« -30 % »), pas un élément.
    if rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    Some(rest.trim())
}

/// Découpe une description du profil en intro et éléments.
///
/// # Arguments
///
/// * `text` - Description saisie (absente = vide).
///
/// # Returns
///
/// Structure intro + éléments ; lignes vides ignorées, lignes trimées.
pub fn parse_description(text: Option<&str>) -> DescriptionList {
    let mut list = DescriptionList::default();
    let Some(text) = text else {
        return list;
    };
    // `split('\n')` puis `trim` (qui retire un `\r` final) : même découpage
    // que le miroir TS, quelle que soit la fin de ligne.
    for line in text.split('\n').map(str::trim).filter(|line| !line.is_empty()) {
        match item_content(line) {
            Some("") => {}
            Some(content) => list.items.push(vec![content.to_string()]),
            None => match list.items.last_mut() {
                Some(item) => item.push(line.to_string()),
                None => list.intro.push(line.to_string()),
            },
        }
    }
    list
}

impl DescriptionList {
    /// Vrai si la description n'a ni intro ni élément.
    ///
    /// # Returns
    ///
    /// Vrai si vide.
    pub fn is_empty(&self) -> bool {
        self.intro.is_empty() && self.items.is_empty()
    }

    /// Toutes les lignes dans l'ordre (intro, puis chaque élément avec ses
    /// suites), sans symbole : la forme envoyée à la traduction.
    ///
    /// # Returns
    ///
    /// Liste plate des lignes.
    pub fn flat_lines(&self) -> Vec<String> {
        self.intro.iter().chain(self.items.iter().flatten()).cloned().collect()
    }

    /// Reconstruit une description de même forme à partir d'une liste
    /// plate de lignes (par exemple traduites), dans l'ordre de
    /// [`flat_lines`](Self::flat_lines).
    ///
    /// # Arguments
    ///
    /// * `lines` - Lignes de remplacement, autant que `flat_lines()`.
    ///
    /// # Returns
    ///
    /// La description reconstruite, ou `None` si le nombre de lignes diffère.
    pub fn rebuild_from(&self, lines: &[String]) -> Option<DescriptionList> {
        if lines.len() != self.flat_lines().len() {
            return None;
        }
        let (intro, mut rest) = lines.split_at(self.intro.len());
        let mut items = Vec::with_capacity(self.items.len());
        for item in &self.items {
            let (taken, remaining) = rest.split_at(item.len());
            items.push(taken.to_vec());
            rest = remaining;
        }
        Some(DescriptionList { intro: intro.to_vec(), items })
    }

    /// Texte d'une seule ligne pour les prompts (faits du profil) : intro
    /// puis éléments, joints par des espaces, sans symbole.
    ///
    /// # Returns
    ///
    /// Texte aplati (vide si la description est vide).
    pub fn flatten_for_prompt(&self) -> String {
        self.flat_lines().join(" ")
    }
}

#[cfg(test)]
mod tests {
    // Importation des crates externes
    use serde::Deserialize;

    // Importation des modules internes
    use super::*;

    /// Cas partagé avec le miroir TS.
    #[derive(Deserialize)]
    struct Case {
        name: String,
        input: String,
        intro: Vec<String>,
        items: Vec<Vec<String>>,
    }

    /// Table de cas commune au parseur Rust et au parseur TS.
    const CASES: &str = include_str!("../../tests/fixtures/description_list_cases.json");

    #[test]
    fn table_de_cas_commune() {
        let cases: Vec<Case> = serde_json::from_str(CASES).expect("table de cas JSON valide");
        assert!(!cases.is_empty(), "table de cas vide");
        for case in cases {
            let parsed = parse_description(Some(&case.input));
            assert_eq!(parsed.intro, case.intro, "intro du cas « {} »", case.name);
            assert_eq!(parsed.items, case.items, "éléments du cas « {} »", case.name);
        }
    }

    #[test]
    fn description_absente_vide() {
        assert!(parse_description(None).is_empty());
    }

    #[test]
    fn liste_plate_puis_reconstruction() {
        let parsed = parse_description(Some("Intro\n- A\nsuite\n- B"));
        assert_eq!(parsed.flat_lines(), ["Intro", "A", "suite", "B"]);
        let translated: Vec<String> = ["I", "a", "s", "b"].iter().map(|s| s.to_string()).collect();
        let rebuilt = parsed.rebuild_from(&translated).expect("même nombre de lignes");
        assert_eq!(rebuilt.intro, ["I"]);
        assert_eq!(rebuilt.items, [vec!["a".to_string(), "s".to_string()], vec!["b".to_string()]]);
    }

    #[test]
    fn reconstruction_refusee_si_nombre_de_lignes_different() {
        let parsed = parse_description(Some("- A\n- B"));
        assert!(parsed.rebuild_from(&["a".to_string()]).is_none());
    }

    #[test]
    fn aplatissement_pour_prompt_sans_symbole() {
        let parsed = parse_description(Some("Intro\n- A\nsuite\n- B"));
        assert_eq!(parsed.flatten_for_prompt(), "Intro A suite B");
    }
}
