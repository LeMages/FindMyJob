//! Suivi en mémoire de la progression du pipeline de génération de documents
//! (`crate::pipeline::generate_documents`) pour une candidature donnée.
//!
//! Ce store n'est jamais persisté en base : c'est un état éphémère consulté
//! par l'interface via un polling (commande `get_generation_progress`)
//! pendant que la génération tourne en tâche de fond (voir
//! `services::generation`). Une entrée est retirée automatiquement
//! 5 minutes après son passage à `Completed` ou `Failed`.

// Importation des modules standard
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Importation des modules internes
use crate::models::{GenerationProgress, GenerationStatus};

/// Délai avant nettoyage automatique d'une entrée terminée (complétée ou
/// échouée) du store, pour laisser le temps au desktop de récupérer le
/// résultat final avant qu'il ne disparaisse.
const CLEANUP_DELAY: Duration = Duration::from_secs(300);

/// Store partagé, thread-safe, des progressions en cours — une entrée par
/// candidature en cours de génération. Clonable à coût constant (`Arc`),
/// partagé avec les autres champs de `AppState`.
#[derive(Clone, Default)]
pub struct ProgressStore(Arc<Mutex<HashMap<i64, GenerationProgress>>>);

impl ProgressStore {
    /// Crée un store vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Met à jour (ou crée) la progression d'une candidature à une étape
    /// donnée, avec le statut `InProgress`.
    ///
    /// # Arguments
    ///
    /// * `candidature_id` - Identifiant de la candidature suivie.
    /// * `current_step` - Étape courante (1-based, 0 avant la première étape).
    /// * `total_steps` - Nombre total d'étapes.
    /// * `step_label` - Libellé lisible de l'étape courante.
    pub fn update(&self, candidature_id: i64, current_step: u8, total_steps: u8, step_label: &str) {
        let mut store = self.0.lock().expect("verrou ProgressStore empoisonné");
        store.insert(
            candidature_id,
            GenerationProgress {
                candidature_id,
                current_step,
                total_steps,
                step_label: step_label.to_string(),
                status: GenerationStatus::InProgress,
                error: None,
            },
        );
    }

    /// Marque une génération comme terminée avec succès, puis programme le
    /// nettoyage de l'entrée après `CLEANUP_DELAY`.
    pub fn mark_completed(&self, candidature_id: i64) {
        {
            let mut store = self.0.lock().expect("verrou ProgressStore empoisonné");
            if let Some(entry) = store.get_mut(&candidature_id) {
                entry.status = GenerationStatus::Completed;
                entry.current_step = entry.total_steps;
            }
        }
        self.schedule_cleanup(candidature_id);
    }

    /// Marque une génération comme échouée avec le message d'erreur donné,
    /// puis programme le nettoyage de l'entrée après `CLEANUP_DELAY`.
    pub fn mark_failed(&self, candidature_id: i64, error: &str) {
        {
            let mut store = self.0.lock().expect("verrou ProgressStore empoisonné");
            if let Some(entry) = store.get_mut(&candidature_id) {
                entry.status = GenerationStatus::Failed;
                entry.error = Some(error.to_string());
            }
        }
        self.schedule_cleanup(candidature_id);
    }

    /// Récupère la progression courante d'une candidature, si une génération
    /// est en cours ou vient de se terminer (avant nettoyage).
    pub fn get(&self, candidature_id: i64) -> Option<GenerationProgress> {
        let store = self.0.lock().expect("verrou ProgressStore empoisonné");
        store.get(&candidature_id).cloned()
    }

    /// Programme le retrait de l'entrée du store après `CLEANUP_DELAY`, pour
    /// ne pas laisser grossir le store indéfiniment au fil des générations.
    fn schedule_cleanup(&self, candidature_id: i64) {
        let store = self.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(CLEANUP_DELAY).await;
            store.0.lock().expect("verrou ProgressStore empoisonné").remove(&candidature_id);
        });
    }
}
