use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
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

async fn init_db(db_path: &PathBuf) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
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
        .invoke_handler(tauri::generate_handler![db_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
