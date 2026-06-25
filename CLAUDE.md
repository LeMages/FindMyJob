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
| AI | OpenAI-compatible API → Ollama Cloud free tier, configurable base URL |

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

The AI base URL is user-configurable (settings). Prompts are sent as standard OpenAI chat completions. The layer must remain provider-agnostic so switching from Ollama Cloud to a local runtime requires only a base URL change in settings.

## Key conventions

- All user-facing text is in French.
- SQLite file lives in Tauri's app data directory (`tauri::api::path::app_data_dir`), never in the project tree.
- Sensitive config (API keys) is stored via Tauri's secure storage, never in plain files.
- `.env` is gitignored; `.env.example` documents required variables without values.
