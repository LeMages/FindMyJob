# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

FindMyJob is an open-source desktop app (Tauri 2) that helps students track job applications (internships, apprenticeships, CDIs). Data is stored locally in SQLite; only AI inference goes through the cloud.

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 |
| Frontend | Vue 3 + TypeScript + Vite + Tailwind CSS |
| Backend | Rust (Tauri commands) |
| Database | SQLite (local, single-user) |
| AI | Anthropic Claude (Messages API), appelée en HTTP brut via `reqwest` — endpoint et modèle fixes, pas de base URL configurable |

## Commands

```bash
# Install dependencies
npm install

# Dev (launches Tauri window + Vite HMR)
npm run tauri dev

# Production build
npm run tauri build

# Frontend only (Vite dev server, no Tauri window)
npm run dev

# Type-check
npm run type-check   # or: vue-tsc --noEmit

# Lint
npm run lint
```

## Architecture

### IPC bridge (Tauri commands)

The frontend calls Rust via `@tauri-apps/api/core`'s `invoke`. All Tauri commands live in `src-tauri/src/` and are registered with `generate_handler!` in `main.rs` (or `lib.rs` for the Tauri 2 plugin pattern). Each command is a `#[tauri::command]` async fn.

Return types must implement `serde::Serialize`; error types must implement `serde::Serialize + std::error::Error`.

### Frontend structure (`src/`)

- Components use the Composition API (`<script setup lang="ts">`).
- Tailwind utility classes are used directly in templates — no separate CSS files except for global resets in `assets/`.
- Tauri `invoke` calls are wrapped in composables (`src/composables/`) to keep components clean.

### Rust backend (`src-tauri/src/`)

- Database access via `rusqlite` (or `sqlx` with SQLite). Migrations are managed manually or via a migration crate.
- AI calls are plain `reqwest` HTTP calls to the configured OpenAI-compatible endpoint.
- State shared across commands is injected via `tauri::State<T>` with types registered in `.manage()` at startup.

### Data model (planned)

- `Application` — company, position, type (stage/alternance/CDI), contact, notes, cv_version, status
- Status pipeline: `to_apply → applied → followed_up → interview → offer | rejected`

### AI layer

Calls go to Anthropic's Claude Messages API (`crates/core/src/llm/client.rs`), authenticated via `ANTHROPIC_API_KEY`. The model is selectable among `AVAILABLE_MODELS` (stored per-user setting, falls back to `DEFAULT_MODEL`), but the provider itself is not interchangeable — the endpoint, auth headers, and structured-output format are Anthropic-specific.

## Code Conventions

### Rust (`src-tauri/`)

| Aspect | Rule |
|---|---|
| Documentation | `///` for public items, `//!` for modules |
| Linting | `cargo clippy` must pass with zero warnings |
| Imports | Grouped by category, separated by a blank line: std → external crates → internal modules |

### TypeScript/Vue (`src/`)

| Aspect | Rule |
|---|---|
| Documentation | JSDoc for exported functions and composables |
| Linting | `npm run lint` must pass with zero warnings |
| Imports | Grouped by category, separated by a blank line: external packages → components → composables → types |

## Key conventions

- All user-facing text is in French.
- SQLite file lives in Tauri's app data directory (`tauri::api::path::app_data_dir`), never in the project tree.
- Sensitive config (API keys) is stored via Tauri's secure storage, never in plain files.
- `.env` is gitignored; `.env.example` documents required variables without values.
