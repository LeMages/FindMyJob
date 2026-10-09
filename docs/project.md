# FindMyJob — project reference

This document is the technical reference for FindMyJob. It exists so a conversation with an AI assistant (or a new contributor) can resume without re-explaining the project from scratch. It describes the what, the why, and the actual state of the code.

Last updated: 2026-09-17, against the `refactor/cargo-workspace` branch, after the split from a single Tauri+SQLite app into a 3-crate workspace with a headless agent backend.

## 1. Vision

FindMyJob is an open-source desktop application that helps students track job applications (internships, apprenticeships, permanent contracts). The goal is not a plain CRUD tracker but an AI-augmented tool that removes the tedious parts of a job search: entering offer details by hand, tailoring a CV and cover letter, drafting an application message.

Target audience: students looking for internships, French market first. The project started as a personal need (the author is a computer engineering student) and was open-sourced to help people in the same situation.

Guiding principle: think the scope and schema through as completely as possible upfront to avoid churn later. Move gate by gate, validate each foundation before building on top of it.

## 2. Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3.5 + TypeScript 5 + Vite 6, Tailwind CSS 4 via `@tailwindcss/vite` (no `tailwind.config` file) |
| Backend (headless) | Rust, Axum 0.8, `tokio-cron-scheduler` for the offers pipeline |
| Database | PostgreSQL 16 + pgvector (`pgvector/pgvector:pg16`), accessed via sqlx 0.8 (`postgres`, `runtime-tokio-rustls`, `migrate`) |
| Document compilation | `typst-as-lib` 0.16 + `typst-pdf` 0.15 (CV/cover-letter PDF generation, no external `typst` binary) |
| AI | Anthropic Claude Messages API, called in raw HTTP via `reqwest` 0.12 (rustls) — no official Rust SDK |
| Deployment (agent) | Docker Compose (`postgres` + `agent` services), `Dockerfile.agent` |

The Claude model is selectable in settings: `AVAILABLE_MODELS = ["claude-sonnet-4-6", "claude-haiku-4-5-20251001"]`, default `DEFAULT_MODEL = "claude-sonnet-4-6"` (`crates/core/src/llm/client.rs`), resolved per request via `crates/agent/src/db/settings.rs::resolve_llm_model`.

Other notable dependencies: `tauri-plugin-dialog` (native file picker), `dotenvy` (`.env` loading), `base64` (PDF bytes over the API/IPC), `open` (opening folders/URLs from the OS), `scraper` (HTML parsing for offer scraping and extraction), `chrono` (dates), `toml` (agent static config).

No vue-router: navigation is a homemade router via `state.screen` (`crates/desktop/src/stores/app.ts` + `src/App.vue`). No Pinia: the store is a plain exported `reactive` object.

## 3. Workspace architecture

FindMyJob is a Cargo workspace with three members, declared in the root `Cargo.toml`:

```toml
[workspace]
members = ["crates/core", "crates/agent", "crates/desktop/src-tauri"]
```

### `crates/core` — `findmyjob-core`

Pure business logic: models, LLM calls, HTML scraping helpers. No database dependency, no Tauri, no Axum — deliberately so it can be shared unmodified by both the agent (server-side) and the desktop app (client-side).

```
crates/core/src/
  lib.rs
  error.rs        AppError shared by both crates that use core
  docgen/         pure document-pipeline logic: budgets, linter, selection, period/date formatting, text diff
  llm/            Claude client, prompt assembly per pipeline step (templatization, matching, writing, label translation)
  models/         serde DTOs re-exported for both the API and the Tauri commands
  scraping/       HTML fetch (fetch.rs) + text extraction (extract.rs)
```

### `crates/agent` — `findmyjob-agent`

Headless binary crate: an Axum HTTP server that owns all persistence, the offers pipeline (scraping, filtering, scoring), the documents pipeline (Typst generation), and the cron scheduler. This is where the business rules live.

```
crates/agent/src/
  main.rs, config.rs, pipeline.rs, scheduler.rs, security.rs, logging.rs
  docgen/         deterministic document assembly: cv_data/lm_data builders, labels cache, page fitting, trace
  api/            22 route modules + mod.rs (router, auth middleware, CORS, error mapping)
  db/             one file per table + common.rs, config.rs, init.rs
  documents/      Typst compilation, reference templatization, on-disk document storage
  filtering/      mechanical filter + LLM scoring for scraped offers
  scraping/       job board scrapers (wttj.rs today) behind a JobScraper trait
```

### `crates/desktop` — Vue 3 frontend + Tauri shell

The Vue frontend lives at `crates/desktop/` itself; the Tauri Rust backend is nested one level deeper, at `crates/desktop/src-tauri/`.

```
crates/desktop/
  package.json, vite.config.ts, tsconfig*.json
  src/
    views/<view_name>/XxxView.vue      one view = one folder, view-specific components inside
    components/                        components shared across views
    composables/                       reusable view-layer logic (formatting, editable lists)
    stores/app.ts                      global reactive app state
    types/index.ts                     TS DTOs hand-mirrored from crates/core models
  src-tauri/
    src/
      main.rs, lib.rs, error.rs
      api_client.rs                    reqwest-based HTTP client to the agent, bearer-token auth
      commands/                        one #[tauri::command] file per domain, delegates to api_client
      models/mod.rs                    `pub use findmyjob_core::models::*;` — no local duplication
    tauri.conf.json, capabilities/, icons/
```

`crates/desktop/src-tauri/Cargo.toml` has no `sqlx`/`rusqlite` dependency at all — the desktop app never touches a database directly. Its `api_client.rs` doc comment sums up the design: *"HTTP client to the agent's REST API (findmyjob-agent), replacing direct database access from the desktop app."*

## 4. Frontend/backend boundary (thin-client principle)

Guiding principle: **thin client**. Vue is a rendering and interaction layer; the agent is the brain. No business truth is decided in Vue, and the desktop Tauri commands are themselves thin — they only translate a Tauri invocation into an HTTP call to the agent and back.

- **World state (agent)**: everything that is *true* about the data (applications, profile, status transitions, timeline rules). This is the business model — it lives in `crates/agent` and nowhere else.
- **View state (Vue)**: what's on screen right now (active tab, open modal, unsaved form content, selected filter). Stays in Vue. Vue's reactivity (`ref`, `computed`, `watch`) manages view state; that is not business logic and is expected.

**Everything the agent computes, it sends already computed in the DTO. Vue displays, it never recomputes a business value.**

- API responses are display-ready DTOs with derived fields already included, not raw SQL rows.
- If a rule depends on a setting, the computation lives in the agent to stay in a single place.
- The desktop's Tauri commands (`crates/desktop/src-tauri/src/commands/`) are a proxy layer over `api_client::ApiClient` — they do not reimplement any rule the agent already enforces.

### Reference example: adding a timeline event

1. The frontend calls the `get_available_events` Tauri command, which proxies to `GET /api/candidatures/{id}/events` (backed by `crates/agent/src/db/timeline_events.rs`, the sole source of truth for which events are legal in the current status).
2. The user picks an event in `AddEventPopup.vue` and fills in the fields required for that event type.
3. The frontend calls `add_timeline_event`, which proxies to `POST /api/candidatures/{id}/events`. The agent **revalidates** that the event is allowed for the current status (never trusts what the client sent), applies its side effects (creating/updating an interview, changing the status), and writes to `status_history`, all in one transaction.
4. If the agent rejects the event, the frontend surfaces the error without having mutated its local state.

Vue proposes, the agent disposes: Vue never hardcodes the legal transitions, it asks the agent every time via `get_available_events`.

### Type contract (kept manual, not generated)

The desired source of truth for the API contract is the Rust DTOs in `crates/core::models`. In practice, `crates/desktop/src/types/index.ts` is written by hand, mirroring those DTOs manually, with no code generation. This is a known, open decision — not yet worth the churn of introducing `tauri-specta` or `ts-rs` given the API now goes through HTTP rather than direct IPC calls.

## 5. Data model

The database is PostgreSQL 16 with the pgvector extension available (image `pgvector/pgvector:pg16`, run via `docker-compose.yml`). The agent owns the connection pool (`sqlx::PgPool`) exclusively — nothing else touches the database directly.

Migrations live in `db/migrations/postgres/` (sqlx migrate, applied at agent startup via `db::init::init_db`):

- `0001_create_offers.sql` — creates the `vector` extension and the `offers` table.
- `0002_app_schema.sql` — the bulk of the application schema: `users`, `lang`, `projets`, `experiences`, `formations`, `skills`, `sessions`, `candidatures`, `contacts`, `interviews`, `status_history`, `documents`, `emails`, `embeddings`, `settings`, plus a `trigger_set_updated_at()` function attached to most tables.
- `0003_offers_add_description.sql` — adds `offers.description`.
- `0004_cleanup_schema.sql` — drops `candidatures.last_relance`/`next_relance`, `offers.embedding`, and the unused `emails`/`embeddings` tables. The `vector` extension from `0001` stays loaded even though no column uses it anymore.

The full, authoritative schema (tables, columns, enums, foreign keys) is kept in `db/schema.dbml`, generated from the migrations — read it for exact column types and constraints rather than duplicating them here. A rendered diagram sits alongside it at `db/schema.png`.

Table summary:

| Table | Role |
|---|---|
| `users` | Single user's profile (name, contact info, driving license, LinkedIn/GitHub/portfolio links) |
| `lang` | Languages spoken by the user, with proficiency level |
| `formations` | Degrees and schools in the profile |
| `projets` | Personal projects attached to the profile |
| `experiences` | Work experience entries in the profile |
| `skills` | Skills, grouped by a free-text domain |
| `sessions` | A search "session" (context, zone, status) grouping a set of applications, with its own reference CV/cover letter |
| `candidatures` | Core entity — one row per application (company, status, metadata, generated document paths) |
| `status_history` | Unified timeline of events for an application (status changes, follow-ups, interviews, messages, calls, notes) |
| `contacts` | People associated with an application (recruiter, manager…) — read-only today, see §12 |
| `interviews` | Scheduled or completed interviews for an application |
| `documents` | Reference CV / cover letter for a session (file path + cached LLM style analysis) |
| `settings` | Key-value app settings (LLM model, theme) |
| `offers` | Scraped job listings feeding the offers pipeline (source, score, status) — see §7 |

## 6. Application lifecycle

### Statuses (11 values, `candidatures.status`)

`a_postuler`, `postule`, `relance`, `entretien`, `entretien_passe`, `offre`, `refus`, `abandon`, `ghosting`, `accepte`, `offre_refusee`.

`entretien_passe` distinguishes an upcoming interview from one already completed but with no response from the company yet. `offre` is an intermediate status: receiving an offer is not terminal, it can still be accepted (`accepte`) or declined (`offre_refusee`). The negative terminal statuses (`abandon`, `refus`, `ghosting`, `offre_refusee`) move the application to the trash view on the frontend (`TRASHED_STATUSES` in `crates/desktop/src/composables/candidature_display.ts`); `accepte` is terminal but stays visible since it's a positive outcome.

### Timeline event types (15 values, `crates/core/src/models/event_type.rs`)

`creation`, `candidature_envoyee`, `relance`, `entretien_planifie`, `entretien_passe`, `offre_recue`, `refus_recu`, `ghosting`, `desistement`, `accepter_offre`, `decliner_offre`, `message_envoye`, `message_recu`, `appel`, `note`.

Each event has an optional effect on the status (`EventType::changes_status`), e.g. `entretien_planifie` → `entretien`, `accepter_offre` → `accepte`, while `message_envoye`, `appel`, or `note` never change the status. `creation` and `candidature_envoyee` are never offered by `get_available_events` — they're driven by the existing creation and "apply" flows instead. This column has no SQL `CHECK` on purpose: `EventType` is the sole source of validation, in Rust.

### Which events are offered for a given status

`crates/agent/src/db/timeline_events.rs::available_events` is the sole source of truth. A non-terminal application (`a_postuler`, `postule`, `relance`, `entretien`, `entretien_passe`, `offre`) can always receive `message_envoye`, `message_recu`, `appel`, `note`, `desistement`; the remaining events are only offered for a specific subset of statuses (e.g. `relance` from `postule`/`relance`/`entretien_passe`, `accepter_offre`/`decliner_offre` only from `offre`).

### The "apply" flow

Three commands cover the transition from `a_postuler` to `postule` (`crates/agent/src/api/candidatures.rs`), each in three steps: fetch the application detail, export the documents, mark it applied.

- `postuler` (no message) — exports the CV and cover letter to disk.
- `postuler/mail` — has the AI draft an application email (`core::llm::candidature_message::generate_application_email`) before exporting.
- `postuler/message` — has the AI draft a short application message before exporting.

All three then record the `candidature_envoyee` event and refuse (`AppError::Conflict`) if the application is no longer in the `a_postuler` status.

## 7. Offers pipeline (backend-only, no desktop UI yet)

`crates/agent/src/scheduler.rs::run_pipeline` runs three steps in sequence, each logging failures to file and continuing regardless of the previous step's outcome:

1. **Scraping** — `crates/agent/src/scraping/wttj.rs` queries Welcome to the Jungle's public Algolia search API directly (index `wk_cms_jobs_production`), not HTML scraping (the WTTJ site itself sits behind an AWS WAF JS challenge, the Algolia endpoint doesn't). One query per `[[searches]]` entry in `agent.toml`, paginated up to 5 pages of 30 hits. Results are normalized into `ScrapedOffer` and inserted via `db::offers::insert_offers`.
2. **Mechanical filtering** — `crates/agent/src/filtering/mechanical.rs` dismisses (`status = "dismissed"`, `score = -1.0`) offers whose contract type isn't in `agent.toml`'s allow-list, whose title/description contains an excluded keyword, or that are older than `max_days_old`.
3. **LLM scoring** — `crates/agent/src/filtering/scoring.rs` batches the remaining unscored offers (5 at a time) and calls Claude, with a system prompt built from the DB profile (`db::profile::build_profile`) and the session's `zone`/`contexte` (`filtering/profile_summary.rs`), with a structured-output schema (score 0.0–1.0 + reason), persisting `score`/`status = "reviewed"`. One retry on malformed JSON, exponential backoff on HTTP 429.

The scheduler runs the full pipeline once at startup (in a background task, without blocking the HTTP server bind) and then on a cron schedule via `tokio_cron_scheduler`, expression `"0 0 12,18 * * *"` (every day at noon and 6pm) — hardcoded in `scheduler.rs`, not configurable through `agent.toml` or an env var.

`agent.toml` (path from `AGENT_CONFIG_PATH`, loaded once at startup, not hot-reloaded) drives *what* gets searched/filtered/scored:

```toml
[filters]
contract_types = [...]    # allow-list, case-insensitive substring match
exclude_keywords = [...]
max_days_old = 60

[[searches]]               # repeatable
keywords = "..."
location = "..."
country_code = "..."
```

The API exposes the resulting queue (`GET /api/offers`, `GET /api/offers/{id}`, `POST /api/offers/{id}/dismiss`, `POST /api/offers/{id}/applied`), but as of this writing there is no corresponding Tauri command, no `invoke()` call, and no TypeScript type anywhere in `crates/desktop` — the pipeline is fully implemented and running, with no way to see its output from the desktop app yet.

## 8. Documents pipeline (CV / cover letter tailoring)

Each session owns one reference CV and one reference cover letter, stored under `documents/sessions/<session_id>/{cv,lettre_motivation}/` on the agent's `DATA_DIR`. The user's reference `.typ` is never modified: it serves as a style and layout model only. **The profile in the database is the only source of facts** — no value from the reference (name, company, address, figures) ever appears in a generated document.

### 8.1 Templatization, at upload

- **Saving a reference** (`documents/storage.rs::save_reference`) compiles the `.typ`, then converts it into a template (`documents/templatize.rs::templatize`, one Claude call with structured output, `core::llm::templatize`). Both happen before any disk write: a failure writes nothing. The version is stored under `<doc_type>/versions/<version_id>/` as `reference.{typ,pdf}`, `template.typ` and `example_data.json`, tracked in `search.document_versions` (`template_path`, `example_data_path`, `page_count`); `search.documents` points to the current version. Versions can be restored (`documents/reference_versions.rs`), and `POST /api/sessions/{id}/documents/{cv|lm}/templatize` re-templatizes the current reference without a new upload.
- **`template.typ`** is the same document with every content value read from a `data.json` file following a canonical schema (`core::models::doc_data`: `CvData`, `LmData`, identical for every user; fixed texts such as section titles or the closing formula live in `labels`). Lists are rendered with loops, an empty optional field hides its whole line without orphan separators, values are inserted as strings (never evaluated as markup), and the letter date comes from the data.
- **`example_data.json`** is the reference's content in that schema, plus `detected_lang` and, for a CV, `period_style` (month or year per section). It is the style model and the source of the length budgets — never a source of facts.
- **Validation** (`documents/templatize.rs`): the template must load `data.json` and must not use `eval`, `datetime.today` or `read`; rendered with `example_data`, it must not exceed the reference's page count and must render the same words as the reference, except the text the model declares it dropped (`dropped_text`, elements with no field in the schema — never a dated CV entry) and a date computed by the reference; rendered with empty data, it must compile and show no word (proof that nothing is hard-coded). Failures are sent back as feedback (compiler errors with the offending line, missing/extra words), 2 retries at most, then the upload fails with the last problems. `page_count` is the page count of the template rendered with its example data.
- A reference is **ready** (`references_ready`, `DocumentsStatus`) once its template exists.

### 8.2 Generation, per application

`crates/agent/src/pipeline.rs::generate_documents`, triggered from `POST /api/candidatures`:

1. Offer analysis (`core::llm::offer::analyze_offer`); its `language` is the documents' language.
2. Matching (`core::llm::matching::match_profile`), then selection (`core::docgen::selection::select`): every experience is kept, plus the N most relevant projects (N = number of projects in the reference). `highlights` / `keywords_covered` / `match_type` steer what to say; they are not text to copy.
3. Budgets (`core::docgen::budgets`) from `example_data`: exact element counts (paragraphs, projects, skill groups, bullets per entry by rank), word counts at ±10 %, page count.
4. Labels in the offer's language: the reference's own labels, or a Claude translation cached per reference version and language (`docgen/labels_cache.rs`, `labels.<lang>.{cv,lm}.json`).
5. Writing, CV and letter in parallel (`tokio::try_join!`): `core::llm::cv_writing::write_cv` (title, summary, bullets per selected entry id, skill groups) and `core::llm::lm_writing::write_lm` (subject, paragraphs). Prompts carry the offer analysis, the selected profile facts, the reference's prose clearly delimited as a style model, the budgets and the target language. Each output is reviewed (`core::llm::writing_loop`): blocking validation (ids match the selection, every skill exists in the profile, no empty field) plus the linter (`core::docgen::lint`: em/en dashes, Markdown bold, backticks, forbidden phrases from `core::docgen::forbidden_phrases` unless present in the reference, budgets). Violations trigger a new call with the previous output and the problem list, 2 retries at most; after the last one, remaining dashes are replaced mechanically and other lint violations are logged without blocking.
6. Deterministic assembly (`docgen/build_cv_data.rs`, `docgen/build_lm_data.rs`): identity, titles, companies, periods (`core::docgen::period_format`, in the reference's style), education, languages, interests and recipient (company, location, contact — only what the database holds, `docgen/recipient.rs`) come from Rust; prose comes from step 5.
7. Compilation of `template.typ` with the data (`docgen/page_fit.rs`): if the PDF exceeds the reference's page count, writing is re-run with budgets reduced by 15 % (cumulative), twice at most, then the document is kept with a warning.

No LLM produces Typst during generation. **Compilation** uses `typst-as-lib` (`documents/typst.rs`) inside `spawn_blocking`, serving the data as a virtual `data.json` file, with fonts from the system, `typst-kit` embedded fonts and fonts imported via `POST /api/fonts`.

**Artifacts**: `.../artefact/<candidature_id>.pdf` and `.data.json` (the template version is `candidatures.{cv,lm}_reference_version_id`). **Trace**: `documents/sessions/<session_id>/debug/<candidature_id>/` holds `offer_analysis.json`, `matching.json`, `selection.json`, `budgets.json`, `cv_content.json`, `lm_content.json`, `lint_report.json` and `attempts.json`.

## 9. AI layer

### HTTP client (`crates/core/src/llm/client.rs`)

Raw HTTP calls to Anthropic's Messages API (`https://api.anthropic.com/v1/messages`, header `anthropic-version: 2023-06-01`) via `reqwest` — there is no official Rust SDK for the Anthropic API. The key is read from `ANTHROPIC_API_KEY` on every call; on the agent side, `crates/agent/src/security.rs` is the sole writer/reader-of-record (backed by `.env` in development, mutable at runtime through `PUT /api/settings/api-key`).

- `thinking` is explicitly disabled on every request: the pipeline is a single-turn structured generation, and extended thinking shares the same output token budget as the visible response — leaving it on by default truncated responses before any useful content.
- Structured outputs (JSON Schema) are used wherever a schema is provided, to guarantee a strictly conformant response (offer analysis, matching, templatization, CV/LM writing, label translation). The API caps the compiled grammar size: the templatization schema for a CV sits close to that cap, so fixed-value fields are plain strings checked in Rust rather than `enum`s.
- `MAX_TOKENS = 8192` by default; templatization (full template + example data in one response) uses 16384.
- Two explicit failure cases are turned into a clear `AppError::Llm` rather than a downstream JSON-parsing error: `stop_reason == "refusal"` (safety classifier) and `stop_reason == "max_tokens"` (truncated response).

### Application message drafting (`core::llm::candidature_message`)

Two functions (`generate_application_email`, `generate_application_message`), plain text ready to paste, one Claude call each, no structured output. Shared rules: only rephrase what already exists in the profile, no placeholders, no markdown, minimal signature (first + last name only).

## 10. REST API (`crates/agent/src/api/`)

All routes are mounted under `/api` (`crates/agent/src/api/mod.rs`), protected by a bearer-token middleware (`API_TOKEN` env var) except `GET /api/system/health`. If `API_TOKEN` is unset, the middleware lets every request through (dev-mode warning printed at startup). CORS is permissive unless `CORS_ALLOWED_ORIGINS` restricts it to a comma-separated origin list.

| Module | Routes (summary) |
|---|---|
| `users` | user existence check, create, get current, update description/info |
| `profile` | aggregated profile (all sub-entities in one response) |
| `experiences` / `formations` / `langues` / `projets` / `skills` | create / update / delete per sub-entity |
| `sessions` | list / create / update / delete |
| `candidatures` | list by session, create, get, update, delete (à postuler only), apply (no message / by mail / by message) |
| `contacts` | list for an application (read-only, no create route yet) |
| `status_history` | history for an application |
| `timeline_events` | list available events, add an event |
| `interviews` | pending interviews for an application, all interviews for a session |
| `offers` | list, get, dismiss, mark applied |
| `scraping` | on-demand extraction from a URL or pasted text |
| `documents` | save/fetch CV & cover letter references, compile to PDF, inspect/read any document file |
| `fonts` | list / import / delete a server-filesystem font family |
| `settings` | Anthropic API key status/set/delete, LLM model, theme, connection test |
| `system` | health check (unauthenticated) |

## 11. Desktop ↔ agent bridge

`crates/desktop/src-tauri/src/api_client.rs` implements a small `reqwest`-based `ApiClient` (GET/POST/PUT/PATCH/DELETE + no-content variants), injected as `tauri::State<ApiClient>`, carrying the bearer token and base URL (`API_URL` env var, e.g. `http://localhost:3000`).

Every file under `crates/desktop/src-tauri/src/commands/` follows the same shape: an `async fn` taking `api: State<'_, ApiClient>`, calling the matching `/api/...` route, and returning the deserialized DTO or a mapped `AppError` — there is no SQL, no business rule, and no direct database access anywhere in the desktop crate. `crates/desktop/src-tauri/src/models/mod.rs` re-exports `findmyjob_core::models::*` rather than duplicating any type.

## 12. Features — implemented, partial, and planned

### Implemented end-to-end

- Onboarding and full profile management (experiences, formations, languages, projects, skills).
- Sessions: CRUD, reference CV/cover letter per session (import, compile, templatization, version history with restore, PDF preview).
- Applications: creation with mandatory AI-generated documents (failure blocks creation), table view per session, editable detail while `a_postuler`, typed timeline, interview management, trash for negative terminal statuses.
- Apply flow: no message / by mail / by message, all exporting documents to disk.
- Offer import from a pasted URL or pasted text, prefilling the new-application form.
- Interview planning (week/month views).
- Settings: LLM model selection, connection test, fonts, theme. The Anthropic API key is configured on the agent only (`.env`), never from the desktop.
- Offers pipeline: scraping (WTTJ/Algolia), mechanical filtering, LLM scoring, scheduled runs — **backend only**, see §7.
- CI/CD: lint + clippy on every PR to `dev`, Conventional Commits, `dev → staging → main` branch flow, semantic-release on `main`.

### Partial

- **Recruiter contacts**: reading works (`GET /api/candidatures/{id}/contacts`), but there is no `POST` route on the agent, and the desktop's "add contact" action (`addRecruiter` in `crates/desktop/src/stores/app.ts`) is an explicit local-only stub — nothing is persisted.
- **Offers pipeline UI**: the backend pipeline described in §7 runs on schedule and populates the `offers` table, but nothing in the desktop app surfaces it yet.

### Planned / out of MVP scope

- A persisted contact-creation flow (agent route + Tauri command + UI), completing the item above.
- A desktop UI for the offers queue (browse, dismiss, mark applied).
- Smart follow-ups: automatic detection of stale applications and a drafted follow-up message (distinct from the existing manual `relance` status/event).
- A standalone profile-to-offer matching feature (project ranking by relevance), distinct from the internal matching step already used inside the tailoring pipeline (`core::llm::matching::match_profile`).
- A provider-agnostic LLM layer (OpenAI-compatible, configurable base URL) to support Ollama Cloud or a local runtime — not implemented; the provider is Anthropic-specific today (see `CLAUDE.md`).
- Multi-device sync, mobile client, mail/calendar integrations.

## 13. Repository, CI/CD, and conventions

### Commit convention

Conventional Commits drive automated versioning: `<type>(<scope>): <description>`. `feat:` bumps minor, `fix:` bumps patch, `feat!:`/`fix!:`/a `BREAKING CHANGE` footer bumps major. `docs:`, `style:`, `refactor:`, `test:`, `chore:` don't trigger a release. Full detail in `docs/commit.md`; pull request conventions in `docs/pull_request.md`.

### Branch flow and CI (as actually configured)

- `feature/*` → PR to `dev` — `.github/workflows/ci.yml`: `frontend` job (`npm run lint` + `npm run build`, working directory `crates/desktop`), `rust` job (`cargo clippy --workspace -- -D warnings` + `cargo test --workspace`, on `ubuntu-22.04`).
- `dev` → PR to `staging`, `staging` → PR to `main` — `.github/workflows/gatekeeper.yml` blocks any other source branch (only `staging` can merge into `main`, only `dev` can merge into `staging`).
- Push to `main` — `.github/workflows/release.yml`: `semantic-release` (`.releaserc.json`) determines the version, then a Tauri build matrix (macOS ARM64/x64, Ubuntu 22.04, Windows) and a GitHub release if a version was computed.

### Local and deployed running

- Agent: `docker compose up -d` runs Postgres (`pgvector/pgvector:pg16`) and the agent (`Dockerfile.agent`) together, or `cargo run -p findmyjob-agent` against a Postgres reachable via `.env`.
- Desktop: `cd crates/desktop && npm install && npm run tauri dev`, pointed at the agent via `API_URL`.

## 14. Known gaps and points of attention

- CI's `rust` job runs `cargo test --workspace` on every PR. Coverage is limited to offer-text conversion and the documents pipeline's pure logic (budgets, linter, selection, periods, text diff, writing validation, `cv_data`/`lm_data` builders, template compilation on a fixture); no test calls the LLM or the database.
- The root `Cargo.toml` still declares the `sqlite` feature on the shared `sqlx` dependency even though no crate uses SQLite anymore (only `postgres` is actually pulled in, by `crates/agent`) — harmless but worth cleaning up.
- The `vector` (pgvector) Postgres extension is loaded by migration `0001` but nothing uses it: the one column that did, `offers.embedding`, was dropped by migration `0004`. Profile-to-offer matching goes through an LLM call today, not vector similarity.
- `db/schema.png` is a static image, not regenerated automatically from `db/schema.dbml` — verify it still matches before relying on it visually.
