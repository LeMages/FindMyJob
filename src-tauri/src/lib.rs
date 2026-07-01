use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow};
use sqlx::Row;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::Manager;

pub struct DbState {
    pub pool: SqlitePool,
    pub db_path: PathBuf,
}

#[derive(serde::Serialize, Clone)]
pub struct CvView {
    pub id: i64,
    pub label: Option<String>,
    pub file_path: String,
    pub pdf_path: Option<String>,
    pub created_at: String,
}

#[derive(serde::Serialize)]
pub struct SessionView {
    pub id: i64,
    pub nom: String,
    pub candidature_count: i64,
    pub has_accepted: bool,
    pub created_at: String,
}

#[derive(serde::Deserialize)]
pub struct CreateSessionInput {
    pub nom: String,
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

#[cfg_attr(debug_assertions, allow(unused_variables))]
fn resolve_cv_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dev-data")
            .join("cv")
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
            .join("cv")
    }
}

#[cfg_attr(debug_assertions, allow(unused_variables))]
fn resolve_data_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dev-data")
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .app_data_dir()
            .expect("failed to resolve app data dir")
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

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Validation(String),
    Conflict(String),
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(m)
            | AppError::Validation(m)
            | AppError::Conflict(m)
            | AppError::NotFound(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CandidatureView {
    pub id: i64,
    pub session_id: Option<i64>,
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub status: String,
    pub source: Option<String>,
    pub type_contrat: Option<String>,
    pub date_candidature: Option<String>,
    pub next_relance: Option<String>,
    pub created_at: String,
}

#[derive(serde::Deserialize)]
pub struct CreateCandidatureInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub localisation: Option<String>,
    pub type_contrat: Option<String>,
    pub source: Option<String>,
    pub remuneration: Option<String>,
    pub url_offre: Option<String>,
    pub description_offre: Option<String>,
    pub notes: Option<String>,
    pub cv_id: Option<i64>,
    pub session_id: Option<i64>,
}

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

#[derive(serde::Serialize)]
pub struct ExperienceView {
    pub id: i64,
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
    pub type_contrat: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateExperienceInput {
    pub poste: String,
    pub entreprise: Option<String>,
    pub lieu: Option<String>,
    pub description: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
    pub type_contrat: Option<String>,
}

#[derive(serde::Serialize)]
pub struct FormationView {
    pub id: i64,
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateFormationInput {
    pub titre: String,
    pub ecole: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ProjetView {
    pub id: i64,
    pub titre: String,
    pub description: Option<String>,
    pub stack: Option<String>,
    pub url: Option<String>,
    pub pour_qui: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateProjetInput {
    pub titre: String,
    pub description: Option<String>,
    pub stack: Option<String>,
    pub url: Option<String>,
    pub pour_qui: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateUserInfoInput {
    pub prenom: Option<String>,
    pub nom: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub permis: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct CreateSkillInput {
    pub nom: String,
    pub domaine: String,
}

#[derive(serde::Deserialize)]
pub struct CreateLangInput {
    pub nom: String,
    pub level: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SkillView {
    pub id: i64,
    pub domaine: String,
    pub nom: String,
}

#[derive(serde::Serialize)]
pub struct LangView {
    pub id: i64,
    pub nom: Option<String>,
    pub level: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ProfileDto {
    pub user: UserView,
    pub experiences: Vec<ExperienceView>,
    pub formations: Vec<FormationView>,
    pub projets: Vec<ProjetView>,
    pub skills: Vec<SkillView>,
    pub langues: Vec<LangView>,
    pub cv_reference: Option<CvView>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn normalize(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    })
}

fn row_to_user(row: &SqliteRow) -> Result<UserView, sqlx::Error> {
    let permis_int: Option<i64> = row.try_get("permis")?;
    Ok(UserView {
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
    })
}

async fn fetch_first_user(pool: &SqlitePool) -> Result<UserView, AppError> {
    let row = sqlx::query(
        "SELECT id, nom, prenom, email, telephone, ville, permis, description, \
         linkedin, github, portfolio, created_at, updated_at \
         FROM users LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    let row = row.ok_or_else(|| AppError::NotFound("Profil introuvable.".to_string()))?;
    row_to_user(&row).map_err(AppError::from)
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
        return Err(AppError::Conflict("Un utilisateur existe déjà.".to_string()));
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
        return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
    }

    let permis: Option<i64> = input.permis.map(|b| if b { 1 } else { 0 });

    sqlx::query(
        "INSERT INTO users \
           (nom, prenom, email, telephone, ville, permis, description, linkedin, github, portfolio) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&nom)
    .bind(&prenom)
    .bind(&email)
    .bind(normalize(input.telephone))
    .bind(normalize(input.ville))
    .bind(permis)
    .bind(normalize(input.description))
    .bind(normalize(input.linkedin))
    .bind(normalize(input.github))
    .bind(normalize(input.portfolio))
    .execute(&state.pool)
    .await?;

    fetch_first_user(&state.pool).await
}

#[tauri::command]
async fn get_user(state: tauri::State<'_, DbState>) -> Result<UserView, AppError> {
    fetch_first_user(&state.pool).await
}

#[tauri::command]
async fn get_profile(state: tauri::State<'_, DbState>) -> Result<ProfileDto, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let exp_rows = sqlx::query(
        "SELECT id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat \
         FROM experiences WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;

    let experiences: Vec<ExperienceView> = exp_rows
        .iter()
        .map(|r| -> Result<ExperienceView, sqlx::Error> {
            Ok(ExperienceView {
                id: r.try_get("id")?,
                poste: r.try_get("poste")?,
                entreprise: r.try_get("entreprise")?,
                lieu: r.try_get("lieu")?,
                description: r.try_get("description")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
                type_contrat: r.try_get("type_contrat")?,
            })
        })
        .collect::<Result<_, _>>()?;

    let skill_rows = sqlx::query(
        "SELECT id, domaine, nom FROM skills WHERE user_id = ? ORDER BY domaine, nom",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;

    let skills: Vec<SkillView> = skill_rows
        .iter()
        .map(|r| -> Result<SkillView, sqlx::Error> {
            Ok(SkillView {
                id: r.try_get("id")?,
                domaine: r.try_get("domaine")?,
                nom: r.try_get("nom")?,
            })
        })
        .collect::<Result<_, _>>()?;

    let form_rows = sqlx::query(
        "SELECT id, titre, ecole, \
                CAST(date_debut AS TEXT) as date_debut, \
                CAST(date_fin   AS TEXT) as date_fin \
         FROM formations WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;

    let formations: Vec<FormationView> = form_rows
        .iter()
        .map(|r| -> Result<FormationView, sqlx::Error> {
            Ok(FormationView {
                id: r.try_get("id")?,
                titre: r.try_get("titre")?,
                ecole: r.try_get("ecole")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
            })
        })
        .collect::<Result<_, _>>()?;

    let proj_rows = sqlx::query(
        "SELECT id, titre, description, stack, url, pour_qui, \
                CAST(date_debut AS TEXT) as date_debut, \
                CAST(date_fin   AS TEXT) as date_fin \
         FROM projets WHERE user_id = ? \
         ORDER BY COALESCE(date_fin, '9999-12-31') DESC, date_debut DESC",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;

    let projets: Vec<ProjetView> = proj_rows
        .iter()
        .map(|r| -> Result<ProjetView, sqlx::Error> {
            Ok(ProjetView {
                id: r.try_get("id")?,
                titre: r.try_get("titre")?,
                description: r.try_get("description")?,
                stack: r.try_get("stack")?,
                url: r.try_get("url")?,
                pour_qui: r.try_get("pour_qui")?,
                date_debut: r.try_get("date_debut")?,
                date_fin: r.try_get("date_fin")?,
            })
        })
        .collect::<Result<_, _>>()?;

    let lang_rows = sqlx::query(
        "SELECT id, nom, level FROM lang WHERE user_id = ? ORDER BY nom",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;

    let langues: Vec<LangView> = lang_rows
        .iter()
        .map(|r| -> Result<LangView, sqlx::Error> {
            Ok(LangView {
                id: r.try_get("id")?,
                nom: r.try_get("nom")?,
                level: r.try_get("level")?,
            })
        })
        .collect::<Result<_, _>>()?;

    let cv_row = sqlx::query(
        "SELECT id, label, file_path, pdf_path, created_at FROM documents \
         WHERE user_id = ? AND type = 'cv' AND is_reference = 1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(uid)
    .fetch_optional(&state.pool)
    .await?;

    let cv_reference: Option<CvView> = cv_row
        .map(|r| -> Result<CvView, sqlx::Error> {
            Ok(CvView {
                id: r.try_get("id")?,
                label: r.try_get("label")?,
                file_path: r.try_get("file_path")?,
                pdf_path: r.try_get("pdf_path")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .transpose()?;

    Ok(ProfileDto { user, experiences, formations, projets, skills, langues, cv_reference })
}

#[tauri::command]
async fn update_description(
    state: tauri::State<'_, DbState>,
    description: Option<String>,
) -> Result<UserView, AppError> {
    let value = normalize(description);
    sqlx::query(
        "UPDATE users SET description = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(&value)
    .execute(&state.pool)
    .await?;
    fetch_first_user(&state.pool).await
}

#[tauri::command]
async fn add_experience(
    state: tauri::State<'_, DbState>,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation(
            "L'intitulé du poste est requis.".to_string(),
        ));
    }

    let type_contrat = normalize(input.type_contrat);
    if let Some(ref tc) = type_contrat {
        if !["stage", "alternance", "interim", "cdd", "cdi"].contains(&tc.as_str()) {
            return Err(AppError::Validation(
                "Type de contrat invalide.".to_string(),
            ));
        }
    }

    let row = sqlx::query(
        "INSERT INTO experiences \
           (user_id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat",
    )
    .bind(uid)
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(&type_contrat)
    .fetch_one(&state.pool)
    .await?;

    Ok(ExperienceView {
        id: row.try_get("id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        lieu: row.try_get("lieu").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn add_projet(
    state: tauri::State<'_, DbState>,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO projets (user_id, titre, description, stack, url, pour_qui, date_debut, date_fin) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, titre, description, stack, url, pour_qui, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(uid)
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .fetch_one(&state.pool)
    .await?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        stack: row.try_get("stack").map_err(AppError::from)?,
        url: row.try_get("url").map_err(AppError::from)?,
        pour_qui: row.try_get("pour_qui").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn delete_projet(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM projets \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Projet introuvable.".to_string()));
    }

    sqlx::query("DELETE FROM projets WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_projet(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateProjetInput,
) -> Result<ProjetView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE projets \
         SET titre = ?, description = ?, stack = ?, url = ?, pour_qui = ?, \
             date_debut = ?, date_fin = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, titre, description, stack, url, pour_qui, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.description))
    .bind(normalize(input.stack))
    .bind(normalize(input.url))
    .bind(normalize(input.pour_qui))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Projet introuvable.".to_string()))?;

    Ok(ProjetView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        stack: row.try_get("stack").map_err(AppError::from)?,
        url: row.try_get("url").map_err(AppError::from)?,
        pour_qui: row.try_get("pour_qui").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn get_cv_reference(
    state: tauri::State<'_, DbState>,
) -> Result<Option<CvView>, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let row = sqlx::query(
        "SELECT id, label, file_path, pdf_path, created_at FROM documents \
         WHERE user_id = ? AND type = 'cv' AND is_reference = 1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;

    row.map(|r| -> Result<CvView, AppError> {
        Ok(CvView {
            id: r.try_get("id").map_err(AppError::from)?,
            label: r.try_get("label").map_err(AppError::from)?,
            file_path: r.try_get("file_path").map_err(AppError::from)?,
            pdf_path: r.try_get("pdf_path").map_err(AppError::from)?,
            created_at: r.try_get("created_at").map_err(AppError::from)?,
        })
    })
    .transpose()
}

#[tauri::command]
async fn save_cv(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    filename: String,
    content: String,
) -> Result<CvView, AppError> {
    if !filename.to_lowercase().ends_with(".typ") {
        return Err(AppError::Validation(
            "Seuls les fichiers .typ sont acceptés.".to_string(),
        ));
    }

    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let cv_dir = resolve_cv_dir(&app);
    let refs_dir = cv_dir.join("references");
    std::fs::create_dir_all(&refs_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier cv : {}", e)))?;

    // Backup the existing reference CV (if any)
    let existing = sqlx::query(
        "SELECT id, label, file_path, pdf_path FROM documents \
         WHERE user_id = ? AND type = 'cv' AND is_reference = 1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(uid)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(ref row) = existing {
        let old_id: i64 = row.try_get("id").map_err(AppError::from)?;
        let old_file_path: String = row.try_get("file_path").map_err(AppError::from)?;
        let old_pdf_path: Option<String> = row.try_get("pdf_path").map_err(AppError::from)?;
        let old_label: Option<String> = row.try_get("label").map_err(AppError::from)?;

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Copy .typ to backup location
        let backup_typ_path = refs_dir.join(format!("cv_backup_{}.typ", ts));
        if std::path::Path::new(&old_file_path).exists() {
            std::fs::copy(&old_file_path, &backup_typ_path).map_err(|e| {
                AppError::Validation(format!("Sauvegarde de l'ancien CV échouée : {}", e))
            })?;
        }
        let backup_typ_str = backup_typ_path.to_string_lossy().to_string();

        // Copy PDF to backup location (non-fatal if absent)
        let backup_pdf_str: Option<String> = if let Some(ref opdf) = old_pdf_path {
            let backup_pdf_path = refs_dir.join(format!("cv_backup_{}.pdf", ts));
            if std::path::Path::new(opdf).exists() {
                std::fs::copy(opdf, &backup_pdf_path).ok();
            }
            Some(backup_pdf_path.to_string_lossy().to_string())
        } else {
            None
        };

        // Create the backup document row
        let backup_row = sqlx::query(
            "INSERT INTO documents (user_id, type, label, file_path, pdf_path, is_reference) \
             VALUES (?, 'cv', ?, ?, ?, 0) \
             RETURNING id",
        )
        .bind(uid)
        .bind(&old_label)
        .bind(&backup_typ_str)
        .bind(&backup_pdf_str)
        .fetch_one(&state.pool)
        .await?;
        let backup_id: i64 = backup_row.try_get("id").map_err(AppError::from)?;

        // Redirect every candidature that linked to the old reference → backup
        sqlx::query(
            "UPDATE candidatures SET cv_id = ? WHERE cv_id = ? AND user_id = ?",
        )
        .bind(backup_id)
        .bind(old_id)
        .bind(uid)
        .execute(&state.pool)
        .await?;

        // Remove the superseded reference row
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(old_id)
            .execute(&state.pool)
            .await?;
    }

    // Write the new .typ file
    let dest_path = cv_dir.join("cv_reference.typ");
    std::fs::write(&dest_path, content.as_bytes()).map_err(|e| {
        AppError::Validation(format!("Écriture du fichier impossible : {}", e))
    })?;
    let dest_str = dest_path.to_string_lossy().to_string();

    // Compile to PDF
    let root = cv_dir.clone();
    let pdf_bytes = tauri::async_runtime::spawn_blocking(move || compile_to_pdf(content, root))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    let pdf_dest = cv_dir.join("cv_reference.pdf");
    std::fs::write(&pdf_dest, &pdf_bytes).map_err(|e| {
        AppError::Validation(format!("Écriture du PDF impossible : {}", e))
    })?;
    let pdf_str = pdf_dest.to_string_lossy().to_string();

    let row = sqlx::query(
        "INSERT INTO documents (user_id, type, label, file_path, pdf_path, is_reference) \
         VALUES (?, 'cv', ?, ?, ?, 1) \
         RETURNING id, label, file_path, pdf_path, created_at",
    )
    .bind(uid)
    .bind(&filename)
    .bind(&dest_str)
    .bind(&pdf_str)
    .fetch_one(&state.pool)
    .await?;

    Ok(CvView {
        id: row.try_get("id").map_err(AppError::from)?,
        label: row.try_get("label").map_err(AppError::from)?,
        file_path: row.try_get("file_path").map_err(AppError::from)?,
        pdf_path: row.try_get("pdf_path").map_err(AppError::from)?,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

#[tauri::command]
fn open_url(url: String) -> Result<(), AppError> {
    open::that(url).map_err(|e| AppError::Validation(e.to_string()))
}

fn compile_to_pdf(content: String, root: PathBuf) -> Result<Vec<u8>, String> {
    use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

    let engine = TypstEngine::builder()
        .main_file(content.as_str())
        .with_file_system_resolver(root)
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(true)
                .include_embedded_fonts(true),
        )
        .build();

    let doc = engine
        .compile()
        .output
        .map_err(|e| format!("Erreur de compilation Typst : {:?}", e))?;

    typst_pdf::pdf(&doc, &Default::default())
        .map_err(|e| format!("Erreur de génération PDF : {:?}", e))
}

#[tauri::command]
async fn compile_cv(state: tauri::State<'_, DbState>) -> Result<String, AppError> {
    let user = fetch_first_user(&state.pool).await?;

    let row = sqlx::query(
        "SELECT file_path FROM documents \
         WHERE user_id = ? AND type = 'cv' AND is_reference = 1 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Aucun CV de référence trouvé.".to_string()))?;
    let file_path: String = row.try_get("file_path").map_err(AppError::from)?;

    let path = std::path::PathBuf::from(&file_path);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Validation(format!("Impossible de lire le fichier CV : {}", e)))?;
    let root = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));

    let pdf_bytes = tauri::async_runtime::spawn_blocking(move || compile_to_pdf(content, root))
        .await
        .map_err(|e| AppError::Validation(format!("Erreur interne : {}", e)))?
        .map_err(AppError::Validation)?;

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

#[tauri::command]
async fn update_user_info(
    state: tauri::State<'_, DbState>,
    input: UpdateUserInfoInput,
) -> Result<UserView, AppError> {
    let email = normalize(input.email);
    if let Some(ref e) = email {
        if !e.contains('@') {
            return Err(AppError::Validation("L'e-mail doit contenir un @.".to_string()));
        }
    }

    let permis: Option<i64> = input.permis.map(|b| if b { 1 } else { 0 });

    sqlx::query(
        "UPDATE users SET \
           prenom    = COALESCE(?, prenom), \
           nom       = COALESCE(?, nom), \
           email     = COALESCE(?, email), \
           telephone = ?, \
           permis    = ?, \
           updated_at = CURRENT_TIMESTAMP \
         WHERE id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(normalize(input.prenom))
    .bind(normalize(input.nom))
    .bind(email)
    .bind(normalize(input.telephone))
    .bind(permis)
    .execute(&state.pool)
    .await?;

    fetch_first_user(&state.pool).await
}

#[tauri::command]
async fn add_skill(
    state: tauri::State<'_, DbState>,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let nom = input.nom.trim().to_string();
    let domaine = input.domaine.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la compétence est requis.".to_string()));
    }
    if domaine.is_empty() {
        return Err(AppError::Validation("Le domaine est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO skills (user_id, nom, domaine) VALUES (?, ?, ?) \
         RETURNING id, nom, domaine",
    )
    .bind(uid)
    .bind(&nom)
    .bind(&domaine)
    .fetch_one(&state.pool)
    .await?;

    Ok(SkillView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        domaine: row.try_get("domaine").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn delete_skill(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM skills \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Compétence introuvable.".to_string()));
    }

    sqlx::query("DELETE FROM skills WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_skill(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateSkillInput,
) -> Result<SkillView, AppError> {
    let nom = input.nom.trim().to_string();
    let domaine = input.domaine.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la compétence est requis.".to_string()));
    }
    if domaine.is_empty() {
        return Err(AppError::Validation("Le domaine est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE skills SET nom = ?, domaine = ?, updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, nom, domaine",
    )
    .bind(&nom)
    .bind(&domaine)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Compétence introuvable.".to_string()))?;

    Ok(SkillView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        domaine: row.try_get("domaine").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn add_lang(
    state: tauri::State<'_, DbState>,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO lang (user_id, nom, level) VALUES (?, ?, ?) \
         RETURNING id, nom, level",
    )
    .bind(uid)
    .bind(&nom)
    .bind(normalize(input.level))
    .fetch_one(&state.pool)
    .await?;

    Ok(LangView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        level: row.try_get("level").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn delete_lang(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM lang \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Langue introuvable.".to_string()));
    }

    sqlx::query("DELETE FROM lang WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_lang(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateLangInput,
) -> Result<LangView, AppError> {
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la langue est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE lang SET nom = ?, level = ? \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, nom, level",
    )
    .bind(&nom)
    .bind(normalize(input.level))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Langue introuvable.".to_string()))?;

    Ok(LangView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        level: row.try_get("level").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn add_formation(
    state: tauri::State<'_, DbState>,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let uid = user.id;

    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO formations (user_id, titre, ecole, date_debut, date_fin) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, titre, ecole, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(uid)
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .fetch_one(&state.pool)
    .await?;

    Ok(FormationView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        ecole: row.try_get("ecole").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn delete_formation(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM formations \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Formation introuvable.".to_string()));
    }

    sqlx::query("DELETE FROM formations WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_formation(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateFormationInput,
) -> Result<FormationView, AppError> {
    let titre = input.titre.trim().to_string();
    if titre.is_empty() {
        return Err(AppError::Validation("Le titre est requis.".to_string()));
    }

    let row = sqlx::query(
        "UPDATE formations \
         SET titre = ?, ecole = ?, date_debut = ?, date_fin = ?, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, titre, ecole, \
                   CAST(date_debut AS TEXT) as date_debut, \
                   CAST(date_fin   AS TEXT) as date_fin",
    )
    .bind(&titre)
    .bind(normalize(input.ecole))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Formation introuvable.".to_string()))?;

    Ok(FormationView {
        id: row.try_get("id").map_err(AppError::from)?,
        titre: row.try_get("titre").map_err(AppError::from)?,
        ecole: row.try_get("ecole").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn delete_experience(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM experiences \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Expérience introuvable.".to_string()));
    }

    sqlx::query("DELETE FROM experiences WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_experience(
    state: tauri::State<'_, DbState>,
    id: i64,
    input: CreateExperienceInput,
) -> Result<ExperienceView, AppError> {
    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation(
            "L'intitulé du poste est requis.".to_string(),
        ));
    }

    let type_contrat = normalize(input.type_contrat);
    if let Some(ref tc) = type_contrat {
        if !["stage", "alternance", "interim", "cdd", "cdi"].contains(&tc.as_str()) {
            return Err(AppError::Validation(
                "Type de contrat invalide.".to_string(),
            ));
        }
    }

    let row = sqlx::query(
        "UPDATE experiences \
         SET poste = ?, entreprise = ?, lieu = ?, description = ?, \
             date_debut = ?, date_fin = ?, type_contrat = ?, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1) \
         RETURNING id, poste, entreprise, lieu, description, date_debut, date_fin, type_contrat",
    )
    .bind(&poste)
    .bind(normalize(input.entreprise))
    .bind(normalize(input.lieu))
    .bind(normalize(input.description))
    .bind(normalize(input.date_debut))
    .bind(normalize(input.date_fin))
    .bind(&type_contrat)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Expérience introuvable.".to_string()))?;

    Ok(ExperienceView {
        id: row.try_get("id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        lieu: row.try_get("lieu").map_err(AppError::from)?,
        description: row.try_get("description").map_err(AppError::from)?,
        date_debut: row.try_get("date_debut").map_err(AppError::from)?,
        date_fin: row.try_get("date_fin").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn create_candidature(
    state: tauri::State<'_, DbState>,
    input: CreateCandidatureInput,
) -> Result<CandidatureView, AppError> {
    let user = fetch_first_user(&state.pool).await?;

    let poste = input.poste.trim().to_string();
    if poste.is_empty() {
        return Err(AppError::Validation("L'intitulé du poste est requis.".to_string()));
    }

    let entreprise = normalize(input.entreprise)
        .ok_or_else(|| AppError::Validation("L'entreprise est requise.".to_string()))?;

    let localisation = normalize(input.localisation)
        .ok_or_else(|| AppError::Validation("Le lieu est requis.".to_string()))?;

    let source = normalize(input.source)
        .ok_or_else(|| AppError::Validation("La plateforme de l'offre est requise.".to_string()))?;

    let description_offre = normalize(input.description_offre)
        .ok_or_else(|| AppError::Validation("Le contenu de l'offre est requis.".to_string()))?;

    let type_contrat = normalize(input.type_contrat)
        .ok_or_else(|| AppError::Validation("Le type de contrat est requis.".to_string()))?;
    let type_contrat_lower = type_contrat.to_lowercase();
    if !["stage", "alternance", "cdi", "cdd", "interim", "freelance"].contains(&type_contrat_lower.as_str()) {
        return Err(AppError::Validation("Type de contrat invalide.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO candidatures \
           (user_id, poste, entreprise, localisation, type, source, remuneration, \
            url_offre, description_offre, notes, cv_id, session_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, session_id, poste, entreprise, localisation, status, source, \
                   type AS type_contrat, \
                   CAST(date_candidature AS TEXT) AS date_candidature, \
                   CAST(next_relance     AS TEXT) AS next_relance, \
                   created_at",
    )
    .bind(user.id)
    .bind(&poste)
    .bind(&entreprise)
    .bind(&localisation)
    .bind(&type_contrat_lower)
    .bind(&source)
    .bind(normalize(input.remuneration))
    .bind(normalize(input.url_offre))
    .bind(&description_offre)
    .bind(normalize(input.notes))
    .bind(input.cv_id)
    .bind(input.session_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(CandidatureView {
        id: row.try_get("id").map_err(AppError::from)?,
        session_id: row.try_get("session_id").map_err(AppError::from)?,
        poste: row.try_get("poste").map_err(AppError::from)?,
        entreprise: row.try_get("entreprise").map_err(AppError::from)?,
        localisation: row.try_get("localisation").map_err(AppError::from)?,
        status: row.try_get("status").map_err(AppError::from)?,
        source: row.try_get("source").map_err(AppError::from)?,
        type_contrat: row.try_get("type_contrat").map_err(AppError::from)?,
        date_candidature: row.try_get("date_candidature").map_err(AppError::from)?,
        next_relance: row.try_get("next_relance").map_err(AppError::from)?,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn upload_candidature_cv(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    filename: String,
    content: String,
) -> Result<CvView, AppError> {
    if !filename.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Seuls les fichiers PDF sont acceptés.".to_string(),
        ));
    }

    use base64::Engine as _;
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&content)
        .map_err(|e| AppError::Validation(format!("Contenu invalide : {}", e)))?;

    let user = fetch_first_user(&state.pool).await?;

    let gen_dir = resolve_data_dir(&app).join("documents").join("generated");
    std::fs::create_dir_all(&gen_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dest_path = gen_dir.join(format!("cv_{}_{}", ts, filename));
    std::fs::write(&dest_path, &pdf_bytes)
        .map_err(|e| AppError::Validation(format!("Écriture impossible : {}", e)))?;
    let dest_str = dest_path.to_string_lossy().to_string();

    let row = sqlx::query(
        "INSERT INTO documents (user_id, type, label, file_path, is_reference) \
         VALUES (?, 'cv', ?, ?, 0) \
         RETURNING id, label, file_path, pdf_path, created_at",
    )
    .bind(user.id)
    .bind(&filename)
    .bind(&dest_str)
    .fetch_one(&state.pool)
    .await?;

    Ok(CvView {
        id: row.try_get("id").map_err(AppError::from)?,
        label: row.try_get("label").map_err(AppError::from)?,
        file_path: row.try_get("file_path").map_err(AppError::from)?,
        pdf_path: row.try_get("pdf_path").map_err(AppError::from)?,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn get_cv_reference_dir(app: tauri::AppHandle) -> Result<String, AppError> {
    let cv_dir = resolve_cv_dir(&app);
    std::fs::create_dir_all(&cv_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier cv : {}", e)))?;
    Ok(cv_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn select_candidature_cv(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    path: String,
) -> Result<CvView, AppError> {
    if !path.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Seuls les fichiers PDF sont acceptés.".to_string(),
        ));
    }

    let user = fetch_first_user(&state.pool).await?;

    let gen_dir = resolve_data_dir(&app).join("documents").join("generated");
    std::fs::create_dir_all(&gen_dir)
        .map_err(|e| AppError::Validation(format!("Impossible de créer le dossier : {}", e)))?;

    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "cv.pdf".to_string());

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dest_path = gen_dir.join(format!("cv_{}_{}", ts, filename));
    std::fs::copy(&path, &dest_path)
        .map_err(|e| AppError::Validation(format!("Copie du fichier impossible : {}", e)))?;
    let dest_str = dest_path.to_string_lossy().to_string();

    let row = sqlx::query(
        "INSERT INTO documents (user_id, type, label, file_path, is_reference) \
         VALUES (?, 'cv', ?, ?, 0) \
         RETURNING id, label, file_path, pdf_path, created_at",
    )
    .bind(user.id)
    .bind(&filename)
    .bind(&dest_str)
    .fetch_one(&state.pool)
    .await?;

    Ok(CvView {
        id: row.try_get("id").map_err(AppError::from)?,
        label: row.try_get("label").map_err(AppError::from)?,
        file_path: row.try_get("file_path").map_err(AppError::from)?,
        pdf_path: row.try_get("pdf_path").map_err(AppError::from)?,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn get_candidatures(
    state: tauri::State<'_, DbState>,
    session_id: i64,
) -> Result<Vec<CandidatureView>, AppError> {
    let user = fetch_first_user(&state.pool).await?;

    let rows = sqlx::query(
        "SELECT id, session_id, poste, entreprise, localisation, status, source, \
                type AS type_contrat, \
                CAST(date_candidature AS TEXT) AS date_candidature, \
                CAST(next_relance     AS TEXT) AS next_relance, \
                created_at \
         FROM candidatures \
         WHERE user_id = ? AND session_id = ? \
         ORDER BY created_at DESC",
    )
    .bind(user.id)
    .bind(session_id)
    .fetch_all(&state.pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<CandidatureView, sqlx::Error> {
            Ok(CandidatureView {
                id: r.try_get("id")?,
                session_id: r.try_get("session_id")?,
                poste: r.try_get("poste")?,
                entreprise: r.try_get("entreprise")?,
                localisation: r.try_get("localisation")?,
                status: r.try_get("status")?,
                source: r.try_get("source")?,
                type_contrat: r.try_get("type_contrat")?,
                date_candidature: r.try_get("date_candidature")?,
                next_relance: r.try_get("next_relance")?,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

#[tauri::command]
async fn create_session(
    state: tauri::State<'_, DbState>,
    input: CreateSessionInput,
) -> Result<SessionView, AppError> {
    let user = fetch_first_user(&state.pool).await?;
    let nom = input.nom.trim().to_string();
    if nom.is_empty() {
        return Err(AppError::Validation("Le nom de la session est requis.".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO sessions (user_id, nom) VALUES (?, ?) \
         RETURNING id, nom, created_at",
    )
    .bind(user.id)
    .bind(&nom)
    .fetch_one(&state.pool)
    .await?;

    Ok(SessionView {
        id: row.try_get("id").map_err(AppError::from)?,
        nom: row.try_get("nom").map_err(AppError::from)?,
        candidature_count: 0,
        has_accepted: false,
        created_at: row.try_get("created_at").map_err(AppError::from)?,
    })
}

#[tauri::command]
async fn get_sessions(state: tauri::State<'_, DbState>) -> Result<Vec<SessionView>, AppError> {
    let user = fetch_first_user(&state.pool).await?;

    let rows = sqlx::query(
        "SELECT s.id, s.nom, s.created_at, \
                (SELECT COUNT(*) FROM candidatures c WHERE c.session_id = s.id) AS candidature_count, \
                (SELECT CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END \
                 FROM candidatures c \
                 WHERE c.session_id = s.id AND c.status IN ('offre', 'refus', 'abandon')) AS has_accepted \
         FROM sessions s \
         WHERE s.user_id = ? \
         ORDER BY s.created_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<SessionView, sqlx::Error> {
            let has_accepted_int: i64 = r.try_get("has_accepted").unwrap_or(0);
            Ok(SessionView {
                id: r.try_get("id")?,
                nom: r.try_get("nom")?,
                candidature_count: r.try_get("candidature_count").unwrap_or(0),
                has_accepted: has_accepted_int != 0,
                created_at: r.try_get("created_at")?,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(AppError::from)
}

#[tauri::command]
async fn delete_session(state: tauri::State<'_, DbState>, id: i64) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions WHERE id = ? AND user_id = (SELECT id FROM users LIMIT 1)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if n == 0 {
        return Err(AppError::NotFound("Session introuvable.".to_string()));
    }

    let mut tx = state.pool.begin().await?;

    sqlx::query(
        "DELETE FROM email_attachments \
         WHERE email_id IN (SELECT id FROM emails \
           WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?))",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM emails \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM status_history \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM contacts \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM interviews \
         WHERE candidature_id IN (SELECT id FROM candidatures WHERE session_id = ?)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM candidatures WHERE session_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = resolve_db_path(app.handle());
            let pool = tauri::async_runtime::block_on(init_db(&db_path))
                .map_err(|e| e.to_string())?;
            app.manage(DbState { pool, db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db_status,
            user_exists,
            create_user,
            get_user,
            get_profile,
            update_description,
            add_experience,
            delete_experience,
            update_experience,
            add_formation,
            delete_formation,
            update_formation,
            add_lang,
            delete_lang,
            update_lang,
            get_cv_reference,
            save_cv,
            compile_cv,
            open_url,
            update_user_info,
            add_projet,
            delete_projet,
            update_projet,
            add_skill,
            delete_skill,
            update_skill,
            create_candidature,
            get_candidatures,
            upload_candidature_cv,
            get_cv_reference_dir,
            select_candidature_cv,
            create_session,
            get_sessions,
            delete_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
