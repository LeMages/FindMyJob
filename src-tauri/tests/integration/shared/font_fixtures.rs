//! Polices de fixture (`tests/fixtures/fonts/`, sous-ensembles OFL) copiées
//! dans un dossier temporaire propre au test.

// Importation des modules standard
use std::fs;
use std::path::Path;

// Importation des modules internes
use crate::shared::fake_llm::fixtures_dir;

/// Copie un dossier de `tests/fixtures/fonts/`, sous-dossiers compris.
///
/// # Arguments
///
/// * `name` - Dossier de fixture (`arimo`, `collection`, `libertinus`, `web`).
/// * `dest` - Dossier de destination, créé au besoin.
pub fn copy_font_fixture(name: &str, dest: &Path) {
    copy_dir(&fixtures_dir().join("fonts").join(name), dest);
}

/// Copie récursive d'un dossier (fixtures peu profondes).
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("création de {} : {e}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("lecture de {} : {e}", from.display())).flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap_or_else(|e| panic!("copie de {} : {e}", entry.path().display()));
        }
    }
}
