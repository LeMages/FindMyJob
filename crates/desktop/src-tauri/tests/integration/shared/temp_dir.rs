//! Dossiers temporaires des tests qui touchent au disque. Tous les tests
//! d'intégration tournent dans un même processus : le pid ne suffit plus à
//! séparer deux tests, d'où un compteur propre au processus dans le nom.

// Importation des modules standard
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Compteur des dossiers déjà attribués dans ce processus de test.
static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

/// Crée un dossier temporaire vide, unique par construction.
///
/// # Arguments
///
/// * `name` - Nom lisible du test, repris dans le nom du dossier.
///
/// # Returns
///
/// Chemin `<temp>/findmyjob-<name>-<pid>-<n>` du dossier créé. Un dossier
/// homonyme laissé par un processus précédent (même pid) est vidé.
pub fn unique_temp_dir(name: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("findmyjob-{name}-{}-{n}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}
