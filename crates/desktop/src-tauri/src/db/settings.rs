//! Persistance des paramètres applicatifs simples (clé/valeur) dans la
//! table `settings`. Utilisée notamment pour le modèle Claude sélectionné.

// Importation des crates externes
use sqlx::{SqlitePool, Row};

// Importation des modules internes
use crate::db::common::map_db_err;
use findmyjob_core::error::AppError;

/// Clé du paramètre stockant l'identifiant du modèle Claude sélectionné.
pub const KEY_LLM_MODEL: &str = "llm_model";

/// Clé du paramètre stockant le thème d'affichage choisi.
pub const KEY_THEME: &str = "theme";

/// Valeurs valides pour le paramètre `theme`.
pub const VALID_THEMES: [&str; 3] = ["clair", "sombre", "systeme"];

/// Thème appliqué en l'absence de valeur enregistrée.
pub const DEFAULT_THEME: &str = "systeme";

/// Récupère la valeur brute associée à une clé de paramètre.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `cle` - Clé du paramètre recherché.
///
/// # Returns
///
/// La valeur associée, ou `None` si la clé n'existe pas encore en base.
pub async fn get(pool: &SqlitePool, cle: &str) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT valeur FROM settings WHERE cle = $1")
        .bind(cle)
        .fetch_optional(pool)
        .await
        .map_err(map_db_err)?;

    match row {
        Some(row) => Ok(row.try_get("valeur").map_err(map_db_err)?),
        None => Ok(None),
    }
}

/// Insère ou met à jour la valeur d'un paramètre (upsert par clé).
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `cle` - Clé du paramètre à écrire.
/// * `valeur` - Nouvelle valeur du paramètre.
///
/// # Returns
///
/// `Ok(())` si l'écriture a réussi.
pub async fn set(pool: &SqlitePool, cle: &str, valeur: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (cle, valeur) VALUES ($1, $2) \
         ON CONFLICT (cle) DO UPDATE SET valeur = excluded.valeur",
    )
    .bind(cle)
    .bind(valeur)
    .execute(pool)
    .await
    .map_err(map_db_err)?;

    Ok(())
}

/// Résout le modèle Claude à utiliser : valeur configurée en base, ou
/// `default` si aucune valeur n'est encore enregistrée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
/// * `default` - Modèle de repli si aucune valeur n'est configurée.
///
/// # Returns
///
/// Identifiant du modèle Claude à utiliser pour les appels LLM.
pub async fn resolve_llm_model(pool: &SqlitePool, default: &str) -> Result<String, AppError> {
    Ok(get(pool, KEY_LLM_MODEL).await?.unwrap_or_else(|| default.to_string()))
}

/// Résout le thème d'affichage configuré : valeur en base, ou
/// [`DEFAULT_THEME`] si aucune valeur n'est encore enregistrée.
///
/// # Arguments
///
/// * `pool` - Pool de connexions SQLite.
///
/// # Returns
///
/// Le thème à appliquer (`"clair"`, `"sombre"` ou `"systeme"`).
pub async fn resolve_theme(pool: &SqlitePool) -> Result<String, AppError> {
    Ok(get(pool, KEY_THEME).await?.unwrap_or_else(|| DEFAULT_THEME.to_string()))
}
