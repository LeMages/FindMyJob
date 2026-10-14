//! Abstraction d'un coffre à secret unique (lecture, écriture, suppression),
//! pour pouvoir remplacer le trousseau du système par un faux dans les tests.

// Importation des modules internes
use crate::error::AppError;

/// Coffre contenant un seul secret. Les implémentations ne tracent jamais
/// la valeur et n'en mettent jamais dans leurs messages d'erreur.
pub trait SecretStore: Send + Sync {
    /// Lit le secret.
    ///
    /// # Returns
    ///
    /// La valeur, `None` si aucun secret n'est enregistré, ou
    /// `AppError::Config` si le coffre est inaccessible.
    fn read(&self) -> Result<Option<String>, AppError>;

    /// Enregistre le secret, en remplaçant la valeur précédente.
    ///
    /// # Arguments
    ///
    /// * `value` - Valeur à enregistrer.
    ///
    /// # Returns
    ///
    /// `Ok(())`, ou `AppError::Config` si le coffre est inaccessible.
    fn write(&self, value: &str) -> Result<(), AppError>;

    /// Supprime le secret. Supprimer un secret absent n'est pas une erreur.
    ///
    /// # Returns
    ///
    /// `Ok(())`, ou `AppError::Config` si le coffre est inaccessible.
    fn delete(&self) -> Result<(), AppError>;
}
