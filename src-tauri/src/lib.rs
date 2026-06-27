use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::Row;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

pub struct DbState {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
}

#[cfg_attr(debug_assertions, allow(unused_variables))]
fn resolve_db_path(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dev-data")
            .join("findmyjob.db")
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
            .join("findmyjob.db")
    }
}

async fn init_db(
    db_path: &PathBuf,
) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(data_dir) = db_path.parent() {
        std::fs::create_dir_all(data_dir)?;
        std::fs::create_dir_all(data_dir.join("documents").join("references"))?;
        std::fs::create_dir_all(data_dir.join("documents").join("generated"))?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!("../db/migrations").run(&pool).await?;

    Ok(pool)
}

// ── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Validation(String),
    Conflict(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(m) | AppError::Validation(m) | AppError::Conflict(m) => {
                write!(f, "{}", m)
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct CreateUserInput {
    pub nom: String,
    pub prenom: String,
    pub email: String,
    pub telephone: Option<String>,
    pub ville: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
}

#[derive(serde::Serialize)]
pub struct UserView {
    pub id: i64,
    pub nom: Option<String>,
    pub prenom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub ville: Option<String>,
    pub permis: Option<bool>,
    pub description: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn normalize(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct DbStatusResult {
    db_path: String,
    tables: Vec<String>,
}

#[tauri::command]
async fn db_status(state: tauri::State<'_, DbState>) -> Result<DbStatusResult, String> {
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master \
         WHERE type = 'table' \
           AND name NOT LIKE 'sqlite_%' \
           AND name != '_sqlx_migrations' \
         ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(DbStatusResult {
        db_path: state.db_path.to_string_lossy().to_string(),
        tables,
    })
}

#[tauri::command]
async fn user_exists(state: tauri::State<'_, DbState>) -> Result<bool, AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&state.pool)
        .await?;
    Ok(n != 0)
}

#[tauri::command]
async fn create_user(
    state: tauri::State<'_, DbState>,
    input: CreateUserInput,
) -> Result<UserView, AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&state.pool)
        .await?;
    if n != 0 {
        return Err(AppError::Conflict(
            "Un utilisateur existe déjà.".to_string(),
        ));
    }

    let nom = input.nom.trim().to_string();
    let prenom = input.prenom.trim().to_string();
    let email = input.email.trim().to_string();

    if nom.is_empty() {
        return Err(AppError::Validation("Le nom est requis.".to_string()));
    }
    if prenom.is_empty() {
        return Err(AppError::Validation("Le prénom est requis.".to_string()));
    }
    if email.is_empty() {
        return Err(AppError::Validation("L'e-mail est requis.".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::Validation(
            "L'e-mail doit contenir un @.".to_string(),
        ));
    }

    let telephone = normalize(input.telephone);
    let ville = normalize(input.ville);
    let description = normalize(input.description);
    let linkedin = normalize(input.linkedin);
    let github = normalize(input.github);
    let portfolio = normalize(input.portfolio);
    let permis: Option<i64> = input.permis.map(|b| if b { 1 } else { 0 });

    let id: i64 = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users \
           (nom, prenom, email, telephone, ville, permis, description, linkedin, github, portfolio) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(&nom)
    .bind(&prenom)
    .bind(&email)
    .bind(&telephone)
    .bind(&ville)
    .bind(permis)
    .bind(&description)
    .bind(&linkedin)
    .bind(&github)
    .bind(&portfolio)
    .fetch_one(&state.pool)
    .await?;

    let row = sqlx::query(
        "SELECT id, nom, prenom, email, telephone, ville, permis, description, \
         linkedin, github, portfolio, created_at, updated_at \
         FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    let permis_int: Option<i64> = row.try_get("permis")?;
    let user = UserView {
        id: row.try_get("id")?,
        nom: row.try_get("nom")?,
        prenom: row.try_get("prenom")?,
        email: row.try_get("email")?,
        telephone: row.try_get("telephone")?,
        ville: row.try_get("ville")?,
        permis: permis_int.map(|n| n != 0),
        description: row.try_get("description")?,
        linkedin: row.try_get("linkedin")?,
        github: row.try_get("github")?,
        portfolio: row.try_get("portfolio")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    };

    Ok(user)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = resolve_db_path(app.handle());
            let pool = tauri::async_runtime::block_on(init_db(&db_path))
                .map_err(|e| e.to_string())?;
            app.manage(DbState { pool, db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![db_status, user_exists, create_user])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
