//! Dossier de debug d'une templatisation (mode DEV) : référence source,
//! chaque essai rejeté et la sortie acceptée, avec les appels à Claude qui
//! les ont produits (prompts, réponses, tokens, durée).
//!
//! Le debug ne bloque jamais l'utilisateur : un échec d'écriture est tracé,
//! jamais propagé.

// Importation des modules standard
use std::path::{Path, PathBuf};

// Importation des modules internes
use crate::llm::LlmCallRecord;
use crate::trace_log;

/// Dossier de debug d'une templatisation.
///
/// # Attributes
///
/// * `dir` - Dossier `templatize_debug/<horodatage>` de cette templatisation.
pub struct TemplatizeDebug {
    dir: PathBuf,
}

impl TemplatizeDebug {
    /// Prépare le dossier et y conserve la référence source, pour pouvoir
    /// rejouer la templatisation.
    ///
    /// # Arguments
    ///
    /// * `dir` - Dossier de debug de cette templatisation.
    /// * `reference_typ` - Source Typst de la référence.
    ///
    /// # Returns
    ///
    /// Le dossier de debug.
    pub fn start(dir: &Path, reference_typ: &str) -> Self {
        let debug = Self { dir: dir.to_path_buf() };
        debug.write(&[("reference.typ".to_string(), reference_typ.to_string())]);
        debug
    }

    /// Conserve un essai : ses fichiers, ses problèmes (vides si accepté)
    /// et les appels à Claude qu'il a faits.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Préfixe des fichiers (`attempt_2`, `accepted`…).
    /// * `files` - Fichiers produits par l'essai : `(suffixe, contenu)`.
    /// * `problems` - Problèmes relevés (vide pour un essai accepté).
    /// * `calls` - Appels à Claude de l'essai.
    pub fn save_attempt(&self, prefix: &str, files: &[(&str, &str)], problems: &[String], calls: &[LlmCallRecord]) {
        let mut contents: Vec<(String, String)> =
            files.iter().map(|(suffix, content)| (format!("{prefix}_{suffix}"), content.to_string())).collect();
        contents.push((format!("{prefix}_problems.txt"), problems.join("\n")));
        let calls_json = serde_json::to_string_pretty(calls).unwrap_or_else(|e| format!("appels non sérialisables : {e}"));
        contents.push((format!("{prefix}_calls.json"), calls_json));
        self.write(&contents);
    }

    /// Écrit des fichiers dans le dossier, en traçant un échec.
    fn write(&self, files: &[(String, String)]) {
        let written = std::fs::create_dir_all(&self.dir)
            .and_then(|()| files.iter().try_for_each(|(name, content)| std::fs::write(self.dir.join(name), content)));
        if let Err(e) = written {
            trace_log::log(&format!("[templatize] debug non conservé dans {} : {e}", self.dir.display()));
        }
    }
}
