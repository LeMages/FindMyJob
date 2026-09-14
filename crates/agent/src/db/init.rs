//! Initialisation du pool PostgreSQL et application des migrations.

// Importation des crates externes
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

// Importation des modules internes
use crate::db::config::DbConfig;

/// Ouvre le pool PostgreSQL et applique les migrations en attente.
///
/// # Arguments
///
/// * `config` - Paramètres de connexion PostgreSQL.
///
/// # Returns
///
/// Pool de connexions prêt à l'emploi, migrations appliquées.
pub async fn init_db(config: &DbConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.user)
        .password(&config.password)
        .database(&config.database);
    let pool = PgPoolOptions::new().connect_with(options).await?;

    // crates/agent -> crates -> racine du repo : deux niveaux au-dessus du
    // CARGO_MANIFEST_DIR de ce crate.
    sqlx::migrate!("../../db/migrations/postgres")
        .run(&pool)
        .await?;

    Ok(pool)
}
