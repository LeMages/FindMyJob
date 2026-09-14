//! Configuration de connexion à la base PostgreSQL, lue depuis les
//! variables d'environnement.

/// Paramètres de connexion PostgreSQL.
///
/// Attributes:
///     host: Hôte du serveur PostgreSQL.
///     port: Port du serveur PostgreSQL.
///     user: Utilisateur PostgreSQL.
///     password: Mot de passe PostgreSQL.
///     database: Nom de la base de données.
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    /// Lit la configuration depuis les variables d'environnement
    /// `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_DATABASE`.
    ///
    /// # Returns
    ///
    /// Configuration de connexion prête à l'emploi.
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("DB_HOST").expect("DB_HOST must be set"),
            port: std::env::var("DB_PORT")
                .expect("DB_PORT must be set")
                .parse()
                .expect("DB_PORT must be a valid port number"),
            user: std::env::var("DB_USER").expect("DB_USER must be set"),
            password: std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
            database: std::env::var("DB_DATABASE").expect("DB_DATABASE must be set"),
        }
    }
}
