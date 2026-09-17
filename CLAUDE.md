# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

FindMyJob is an open-source desktop app (Tauri 2) that helps students track job applications (internships, apprenticeships, permanent contracts). It's a Cargo workspace of three crates: a shared logic library, a headless Axum server that owns all persistence and AI pipelines, and the Tauri desktop shell, which is a pure HTTP client of that server. See [`docs/project.md`](docs/project.md) for the exhaustive technical reference — this file is the day-to-day working guide.

## Workspace architecture

```
crates/core     findmyjob-core   — shared business logic: models, LLM calls, HTML scraping. No DB, no Tauri, no Axum dependency.
crates/agent    findmyjob-agent — headless Axum server. Owns the PostgreSQL connection, the offers pipeline (scraping/filtering/scoring), the documents pipeline (Typst generation), the cron scheduler, and every business rule.
crates/desktop  Vue 3 frontend at crates/desktop/, Tauri backend nested at crates/desktop/src-tauri/. Thin HTTP client of the agent — no database access, no business logic beyond proxying.
```

`crates/agent` is the single source of truth for business rules (status transitions, timeline rules, validation). `crates/desktop` never reimplements a rule the agent already enforces — it calls the agent and displays what comes back.

## Where things live

| What | Where |
|---|---|
| Shared DTOs (serde structs used by both agent and desktop) | `crates/core/src/models/*.rs`, one file per domain |
| Claude API client and prompt assembly | `crates/core/src/llm/*.rs` (`client.rs` is the raw HTTP client) |
| Database access (PostgreSQL via sqlx) | `crates/agent/src/db/*.rs`, one file per table |
| REST API routes | `crates/agent/src/api/*.rs`, one file per resource, mounted in `crates/agent/src/api/mod.rs` |
| Offers pipeline (scraping/filtering/scoring) | `crates/agent/src/scraping/`, `crates/agent/src/filtering/`, orchestrated by `crates/agent/src/scheduler.rs` |
| Documents pipeline (CV/LM tailoring + Typst) | `crates/agent/src/pipeline.rs` (orchestration), `crates/agent/src/documents/` (storage + compilation) |
| Tauri commands | `crates/desktop/src-tauri/src/commands/*.rs`, one file per domain, each delegating to `api_client::ApiClient` |
| HTTP client to the agent | `crates/desktop/src-tauri/src/api_client.rs` |
| TypeScript types | `crates/desktop/src/types/index.ts` (hand-mirrored from `crates/core::models`, no codegen) |
| Vue views and components | `crates/desktop/src/views/<view_name>/` (one folder per view) and `crates/desktop/src/components/` (shared) |
| Agent static config (profile summary, search filters) | `agent.toml`, loaded from `AGENT_CONFIG_PATH` |

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

- New migrations go in `db/migrations/postgres/`, numbered sequentially (`NNNN_description.sql`), applied via sqlx migrate at agent startup.
- After adding or changing a migration, update `db/schema.dbml` to match — it's the hand-maintained reference for the full schema (columns, enums, foreign keys).

## Adding a feature end-to-end

For a change that touches the database and needs to reach the UI, follow the chain in order:

1. **Migration** — add a `.sql` file in `db/migrations/postgres/`, update `db/schema.dbml`.
2. **DB module** — add/extend the table's file in `crates/agent/src/db/`.
3. **API route** — add the handler in `crates/agent/src/api/<resource>.rs`, register it in `crates/agent/src/api/mod.rs`.
4. **Tauri command** — add the command in `crates/desktop/src-tauri/src/commands/<resource>.rs`, calling the new route via `api_client::ApiClient`; register it in the `generate_handler!` list in `crates/desktop/src-tauri/src/lib.rs`.
5. **TypeScript type** — mirror the DTO by hand in `crates/desktop/src/types/index.ts`.
6. **Vue component** — wire the `invoke()` call into the relevant view or component.

Skipping a step usually means the feature is backend-only (see the offers pipeline, below) — that's a valid intermediate state, but call it out rather than letting a route silently go unused.

## Running the project locally

```bash
cp .env.example .env      # fill in ANTHROPIC_API_KEY at minimum
docker compose up -d      # PostgreSQL (pgvector image) + the agent
cd crates/desktop
npm install
npm run tauri dev
```

To iterate on the agent alone, without Docker: `cargo run -p findmyjob-agent` against a Postgres reachable via the `DB_*` vars in `.env`. Agent config beyond env vars (search queries, filters, profile summary for scoring) lives in `agent.toml`, loaded from `AGENT_CONFIG_PATH` and **not** hot-reloaded — restart the agent after editing it.

`npm`/`vite`/`tauri` commands must be run from `crates/desktop/`, not the repo root — there is no root `package.json`.

## Points of attention

- **The offers pipeline has no desktop UI yet.** `crates/agent` fully implements scraping (WTTJ via Algolia), mechanical filtering, LLM scoring, and a cron schedule, and exposes `GET/POST /api/offers/*`. Nothing in `crates/desktop` calls it — no Tauri command, no `invoke()`, no TS type. Don't assume a desktop feature exists just because the backend route does.
- **Recruiter contact creation is not persisted.** The agent only exposes `GET /api/candidatures/{id}/contacts` — there is no `POST` route. The desktop's "add contact" action is an explicit local-only stub in `crates/desktop/src/stores/app.ts` (`addRecruiter`). Implementing this for real means the full chain in the section above, not a frontend-only fix.
- **`crates/agent/src/pipeline.rs:169` has a `[DIAG-TEMP]` marker.** CV/cover-letter content generation was deliberately made sequential instead of `tokio::try_join!` to diagnose a concurrency race. Don't revert it to parallel without checking whether that diagnosis concluded.
- **CI's `cargo test` job currently runs against zero tests.** It passes vacuously — don't treat a green CI run as evidence of test coverage.

## What NOT to do

- Don't use `rusqlite` or assume SQLite anywhere — it has been fully removed. The agent uses PostgreSQL via sqlx exclusively.
- Don't look for or add migrations under a `db/migrations/sqlite/` path — it doesn't exist; only `db/migrations/postgres/` is live.
- Don't add business logic to `crates/desktop` (Vue or Tauri commands) that duplicates a rule the agent already enforces — the agent is the single source of truth (see the thin-client principle in `docs/project.md`).
- Don't run `npm`, `vite`, or `tauri` commands from the repo root.
- Don't introduce a configurable AI provider base URL or an OpenAI-compatible abstraction — the provider is deliberately Anthropic-specific today (endpoint, auth headers, and structured-output format are all Claude-specific in `crates/core/src/llm/client.rs`).

## Key conventions

- All user-facing text is in French.
- The agent's data directory (`DATA_DIR`, documents, fonts) and PostgreSQL storage are never inside the project tree — configured via env vars, see `.env.example`.
- Sensitive config (`ANTHROPIC_API_KEY`, `API_TOKEN`) is read from environment variables (`.env` in development), never hardcoded or committed. `.env` is gitignored; `.env.example` documents required variables without values.
