//! Script de build : génération Tauri et suivi des migrations SQL.

/// Dossier des migrations, relatif à `src-tauri/`, embarqué par `sqlx::migrate!`.
const MIGRATIONS_DIR: &str = "../db/migrations/sqlite";

fn main() {
    // `sqlx::migrate!` lit les fichiers SQL à la compilation, mais Cargo ne
    // les suit pas : sans cette ligne, ajouter ou modifier une migration ne
    // recompile pas la crate et le binaire garde les anciennes migrations.
    println!("cargo:rerun-if-changed={MIGRATIONS_DIR}");
    tauri_build::build()
}
