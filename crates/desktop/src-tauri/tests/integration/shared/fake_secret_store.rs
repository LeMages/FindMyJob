//! Faux trousseau en mémoire pour les tests : ne touche jamais au
//! trousseau du système, et compte ses accès pour vérifier qu'un mode
//! donné ne le consulte pas.

// Importation des modules standard
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// Importation des modules internes
use findmyjob_core::error::AppError;
use findmyjob_lib::secrets::store::SecretStore;

/// Trousseau en mémoire.
///
/// # Attributes
///
/// * `value` - Secret enregistré.
/// * `accesses` - Nombre d'appels à `read`, `write` ou `delete`.
#[derive(Default)]
pub struct FakeSecretStore {
    value: Mutex<Option<String>>,
    accesses: AtomicUsize,
}

impl FakeSecretStore {
    /// Trousseau contenant déjà un secret.
    pub fn with(value: &str) -> Self {
        Self { value: Mutex::new(Some(value.to_string())), accesses: AtomicUsize::new(0) }
    }

    /// Secret actuellement enregistré.
    pub fn value(&self) -> Option<String> {
        self.value.lock().unwrap().clone()
    }

    /// Nombre d'accès au trousseau depuis sa création.
    pub fn accesses(&self) -> usize {
        self.accesses.load(Ordering::SeqCst)
    }
}

impl SecretStore for FakeSecretStore {
    fn read(&self) -> Result<Option<String>, AppError> {
        self.accesses.fetch_add(1, Ordering::SeqCst);
        Ok(self.value.lock().unwrap().clone())
    }

    fn write(&self, value: &str) -> Result<(), AppError> {
        self.accesses.fetch_add(1, Ordering::SeqCst);
        *self.value.lock().unwrap() = Some(value.to_string());
        Ok(())
    }

    fn delete(&self) -> Result<(), AppError> {
        self.accesses.fetch_add(1, Ordering::SeqCst);
        *self.value.lock().unwrap() = None;
        Ok(())
    }
}
