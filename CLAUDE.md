# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

FindMyJob is an open-source desktop app (Tauri 2) that helps students track job applications (internships, apprenticeships, permanent contracts). It's a standalone application — one process, no server, no Docker — built as a Cargo workspace of two crates: a pure shared logic library, and the Tauri desktop app, whose Rust side owns the SQLite database, the AI pipelines and every business rule. Offers enter the app only through their link (or pasted text) — there is no automatic offer collection. See [`docs/project.md`](docs/project.md) for the exhaustive technical reference — this file is the day-to-day working guide.

## Workspace architecture

```
crates/core     findmyjob-core — shared business logic: models, LLM calls, offer page fetch and text extraction, diagnostic traces. No DB, no Tauri dependency.
crates/desktop  Vue 3 frontend at crates/desktop/, Tauri backend nested at crates/desktop/src-tauri/. The Rust side owns the SQLite pool, offer extraction by link, the documents pipeline (Typst generation) and every business rule.
```

The Rust side of `crates/desktop` is the single source of truth for business rules (status transitions, timeline rules, validation). The Vue frontend never reimplements a rule — it calls a Tauri command and displays what comes back. Tauri commands are themselves thin: they call one service or one `db` function.

## Where things live

| What | Where |
|---|---|
| Shared DTOs (serde structs crossing the IPC bridge) | `crates/core/src/models/*.rs`, one file per domain |
| Claude API client and prompt assembly | `crates/core/src/llm/*.rs` (`client.rs` is the raw HTTP client; every call takes a `ClaudeTarget` = key + model, `core` never reads the key itself) |
| Anthropic API key (DEV `.env` / PROD system keyring) | `crates/desktop/src-tauri/src/services/api_key.rs` (`ApiKeyAccess::resolve`, the only place that picks the source), `services/claude_target.rs` (key + model for every Claude call), `secrets/` (`SecretStore` trait, keyring implementation) |
| Diagnostic traces (stderr + log file) | `crates/core/src/trace_log.rs` — use `trace_log::log`, never `eprintln!` |
| Database access (SQLite via sqlx) | `crates/desktop/src-tauri/src/db/*.rs`, one file per table; `init.rs` (pool, migrations), `transaction.rs` (`begin_write`) |
| Business logic beyond one DB call | `crates/desktop/src-tauri/src/services/*.rs` (applications, background generation, settings, offer import, sessions) |
| Current time and stored date formats | `crates/desktop/src-tauri/src/clock.rs` — the only place that reads the clock |
| Paths of the data folder | `crates/desktop/src-tauri/src/storage_paths.rs` — the only place that builds or resolves them |
| Startup (`.env`, run mode, log file, folders, pool, cleanup, main window) | `crates/desktop/src-tauri/src/startup/` — the DEV/PROD decision is `run_mode::resolve`, the only place that picks the data, log and WebView folders |
| Offer import by link (URL validation, guarded fetch, JSON-LD then page text + LLM, final cleanup) | `crates/core/src/scraping/` (pure logic), `crates/core/src/llm/scraping_extract.rs`, cascade in `crates/desktop/src-tauri/src/offer_extraction.rs`, entry points in `services/scraping.rs` |
| Capture window (user-driven page capture when the app can't read a page) | `crates/desktop/src-tauri/src/commands/capture.rs` — captures and passes the HTML to `services::scraping::extract_from_html`, no extraction logic |
| Documents pipeline (CV/LM tailoring + Typst) | `crates/desktop/src-tauri/src/pipeline.rs` (orchestration), `docgen/` (deterministic assembly), `documents/` (templatization, storage, versions, compilation, fonts), `crates/core/src/docgen/` (budgets, linter, selection — pure logic) |
| Tauri commands | `crates/desktop/src-tauri/src/commands/*.rs`, one file per domain, registered in `lib.rs` (`generate_handler!`) |
| TypeScript types | `crates/desktop/src/types/index.ts` (hand-mirrored from `crates/core::models`, no codegen) |
| Vue views and components | `crates/desktop/src/views/<view_name>/` (one folder per view) and `crates/desktop/src/components/` (shared) |

## Code conventions

### Rust (`crates/*`)

| Aspect | Rule |
|---|---|
| Documentation | `///` for public items, `//!` for modules |
| Linting | `cargo clippy --workspace -- -D warnings` must pass with zero warnings |
| Imports | Grouped by category, separated by a blank line: std → external crates → internal modules |
| Modules | One file per responsibility; `mod.rs` files contain only `pub mod`/`pub use`, never logic |

### TypeScript/Vue (`crates/desktop/src/`)

| Aspect | Rule |
|---|---|
| Documentation | JSDoc for exported functions and composables |
| Linting | `npm run lint` (run from `crates/desktop/`) must pass with zero warnings |
| Imports | Grouped by category, separated by a blank line: external packages → components → composables → types |

### SQL / migrations

- Migrations go in `db/migrations/sqlite/`, numbered sequentially (`NNNN_description.sql`), embedded in the binary by `sqlx::migrate!` and applied at app startup.
- After adding or changing a migration, update `db/schema.dbml` to match — it's the hand-maintained reference for the full schema (columns, CHECK values, foreign keys).
- Tables are `STRICT`. Timestamps are TEXT RFC 3339 UTC with millisecond precision, produced by `clock.rs` and passed to the writes (one timestamp per transaction); dates are `YYYY-MM-DD`; booleans 0/1. No time defaults, no triggers.
- Write transactions go through `db::transaction::begin_write` (`BEGIN IMMEDIATE`). Never hold one across an LLM call.
- File paths stored in the database are relative to the data folder, with `/` separators (`storage_paths`). A path received from the frontend is resolved with `storage_paths::resolve_untrusted`.
- A UNIQUE constraint that the user can hit gets a check before the write with a clear French message — never surface the raw SQLite error.

## Adding a feature end-to-end

For a change that touches the database and needs to reach the UI, follow the chain in order:

1. **Migration** — add a `.sql` file in `db/migrations/sqlite/`, update `db/schema.dbml`.
2. **DB module** — add/extend the table's file in `crates/desktop/src-tauri/src/db/`, with tests in `crates/desktop/src-tauri/tests/` (in-memory SQLite, `tests/shared/seed.rs`).
3. **Service** — if the logic is more than one DB call (rules, orchestration, files), put it in `crates/desktop/src-tauri/src/services/`.
4. **Tauri command** — add the command in `crates/desktop/src-tauri/src/commands/<resource>.rs`, calling the service or db function via `ctx.state()?`; register it in the `generate_handler!` list in `crates/desktop/src-tauri/src/lib.rs`.
5. **TypeScript type** — mirror the DTO by hand in `crates/desktop/src/types/index.ts`.
6. **Vue component** — wire the `invoke()` call into the relevant view or component.

Skipping a step usually means the feature is backend-only — that's a valid intermediate state, but call it out rather than letting a function silently go unused.

## Running the project locally

```bash
cp .env.example .env      # MODE=DEV for a local dev-data/ folder, ANTHROPIC_API_KEY (read in DEV only)
cd crates/desktop
npm install
npm run tauri dev
```

There is no server and no Docker. At startup the app loads `.env` (app config folder first, then the current directory and its parents — in dev, the repository root), decides the run mode, creates its data folder, opens `findmyjob.db` and applies the migrations.

Run mode (`startup/run_mode.rs`): `MODE` is read from the `.env` that was loaded: no `.env`, `MODE` absent or `MODE=PROD` → PROD; `MODE=DEV` → DEV; any other value is a startup error (case is ignored).

- **PROD** — data folder = Tauri `app_data_dir` (identifier `com.findmyjob.app`; Windows `%APPDATA%\com.findmyjob.app\`: `findmyjob.db`, `documents/`, `polices/`); log file `<app log folder>/findmyjob.log` (Windows `%LOCALAPPDATA%\com.findmyjob.app\logs\`); WebView profiles in the app local folder (`EBWebView/`, `capture_webview/`).
- **DEV** — everything in `dev-data/` next to the `.env` that declared `MODE=DEV` (the repository root): `findmyjob.db`, `documents/`, `polices/`, `logs/findmyjob.log`, `webview/` (main window profile), `capture_webview/`. Nothing goes to the system app folders. `dev-data/` is gitignored and created when missing.

The main window is declared with `"create": false` in `tauri.conf.json` and built in `setup` (`startup/main_window.rs`) so its WebView profile follows the mode.

`npm`/`vite`/`tauri` commands must be run from `crates/desktop/`, not the repo root — there is no root `package.json`.

## Points of attention

- **No automatic offer collection.** Scraping of job boards, filtering, LLM scoring, the scheduler and the Marché tab were removed (they went against several platforms' terms of use); the code is kept at the Git tag `archive/collecte-automatique`. An offer exists only inside its application (`url_offre`, `description_offre`), added by link or pasted text. Don't reintroduce bulk collection from job platforms.
- **No LLM writes Typst during generation.** References are converted once into `template.typ` + `example_data.json` at upload; generation fills a canonical `data.json` from the profile (facts) and LLM-written prose only. The reference is a style model, never a source of facts. See `docs/project.md` §7.
- **Generation runs in the app process.** `services::generation` publishes progress before `create_candidature` returns, then runs the pipeline as a background task. A failed generation deletes the application and its partial files; a generation cut short by closing the app is cleaned up at the next startup (every `a_postuler` application with no document).
- **A reference version is all or nothing.** Its row is inserted in a write transaction before its files are written; any failure rolls back and removes the folder (`documents/storage.rs::persist_version`).
- **Offer import is user-triggered, one page at a time.** No crawl, no retry loop, no proxy/identity rotation/captcha solving. Only a validated public `http(s)` URL is fetched (`core::scraping::url_validation`, DNS filtered by `safe_resolver`). Failures are typed (`ScrapingFailure`, serialized as a code in `AppError::Scraping`) and the UI chooses the message — don't hardcode user-facing advice in the backend. Every description, whatever its path, goes through `core::scraping::finalize::finalize_fields`.
- **The capture window has no IPC access.** The `capture` window is deliberately absent from every capability; its HTML is read from Rust with `eval_with_callback`. Never add it to a capability or give it a `remote` capability, and keep validating the URL before loading it.
- **The API key source follows the run mode.** DEV reads `ANTHROPIC_API_KEY` from the `.env` and never opens the keyring; PROD reads the system keyring (`keyring` crate, `v1` feature only; service = app identifier, account `anthropic_api_key`) and ignores any `.env` key. Code that calls Claude gets its `ClaudeTarget` from `ClaudeTargetSource::resolve`: never read the key elsewhere, never copy it into the environment, never log it, send it to the frontend (only its last 4 characters) or put it in an error. An unavailable keyring is an explicit error, never a fallback to a file. Tests use the in-memory `tests/shared/fake_secret_store.rs`, never the real keyring.
- **Test coverage is narrow.** `cargo test` covers HTML/Markdown-to-text conversion, the offer import's pure logic (synthetic HTML fixtures, no network), the documents pipeline's pure logic, the `db` layer on in-memory SQLite and the file-handling services; nothing tests the LLM calls or the Tauri commands themselves.

## What NOT to do

- Don't reintroduce a local server, an HTTP layer between the frontend and the Rust side, PostgreSQL, or Docker — the server-mode version is archived at the Git tag `archive/agent-postgres`.
- Don't enable another sqlx driver than SQLite, at the workspace or crate level.
- Don't read the clock outside `clock.rs`, and don't build a data-folder path outside `storage_paths.rs`. Don't pick a data, log or WebView folder outside `startup/run_mode.rs` — the rest of the code receives them.
- Don't add database access, Tauri or file-system persistence to `crates/core` — it stays pure.
- Don't add business logic to the Vue frontend or to a Tauri command file — it belongs in `db/` or `services/`.
- Don't run `npm`, `vite`, or `tauri` commands from the repo root.
- Don't introduce a configurable AI provider base URL or an OpenAI-compatible abstraction — the provider is deliberately Anthropic-specific today (endpoint, auth headers, and structured-output format are all Claude-specific in `crates/core/src/llm/client.rs`).

## Key conventions

- All user-facing text is in French.
- The app's data (database, documents, fonts) lives in the Tauri app data folder, never inside the project tree.
- The Anthropic API key is read from `ANTHROPIC_API_KEY` in `.env` in DEV mode, and from the system keyring in PROD mode (set from Paramètres). It is never hardcoded, committed or stored in the database. `.env` is gitignored; `.env.example` documents it without a value.
