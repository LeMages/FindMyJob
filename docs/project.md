# FindMyJob — project reference

This document is the technical reference for FindMyJob. It exists so a conversation with an AI assistant (or a new contributor) can resume without re-explaining the project from scratch. It describes the what, the why, and the actual state of the code.

Last updated: 2026-10-10, after the move to a standalone desktop application: the Axum agent, Docker Compose and PostgreSQL are gone, the Tauri commands call the business logic directly and data lives in a local SQLite file. The server-mode version is kept at the Git tag `archive/agent-postgres`. Earlier, automatic offer collection was removed (tag `archive/collecte-automatique`): offers enter the app only through their link or pasted text (see §9 « Offer import by link »).

## 1. Vision

FindMyJob is an open-source desktop application that helps students track job applications (internships, apprenticeships, permanent contracts). The goal is not a plain CRUD tracker but an AI-augmented tool that removes the tedious parts of a job search: entering offer details by hand, tailoring a CV and cover letter, drafting an application message.

Target audience: students looking for internships, French market first. The project started as a personal need (the author is a computer engineering student) and was open-sourced to help people in the same situation.

Guiding principle: think the scope and schema through as completely as possible upfront to avoid churn later. Move gate by gate, validate each foundation before building on top of it.

## 2. Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3.5 + TypeScript 5 + Vite 6, Tailwind CSS 4 via `@tailwindcss/vite` (no `tailwind.config` file) |
| Backend | Rust, inside the Tauri process (no server) |
| Database | SQLite (bundled), accessed via sqlx 0.8 with the SQLite driver only (`sqlite`, `runtime-tokio`, `migrate`, `chrono`) |
| Document compilation | `typst-as-lib` 0.16 + `typst-pdf` 0.15 (CV/cover-letter PDF generation, no external `typst` binary) |
| AI | Anthropic Claude Messages API, called in raw HTTP via `reqwest` 0.12 (rustls) — no official Rust SDK |

The Claude model is selectable in settings: `AVAILABLE_MODELS = ["claude-sonnet-4-6", "claude-haiku-4-5-20251001"]`, default `DEFAULT_MODEL = "claude-sonnet-4-6"` (`src-tauri/src/llm/client.rs`), resolved per request via `src-tauri/src/db/settings.rs::resolve_llm_model`.

Other notable dependencies: `tauri-plugin-dialog` (native file picker), `dotenvy` (`.env` loading), `keyring` 4 with only its `v1` feature (system keyring for the API key in PROD: Windows Credential Manager, macOS Keychain, Linux Secret Service via zbus with pure-Rust crypto), `base64` (PDF bytes over IPC), `open` (opening folders/URLs from the OS), `scraper` (HTML parsing for offer extraction by link), `chrono` (dates).

No vue-router: navigation is a homemade router via `state.screen` (`src/stores/app.ts` + `src/App.vue`). No Pinia: the store is a plain exported `reactive` object.

## 3. Project layout

FindMyJob follows the standard Tauri layout: the Vue frontend at the repository root, the Rust backend in `src-tauri/`. The root `Cargo.toml` is a workspace with a single member, so `Cargo.lock` and `target/` stay at the root:

```toml
[workspace]
members = ["src-tauri"]
```

The Rust side (crate `findmyjob`, library `findmyjob_lib`) owns the SQLite database, the LLM calls, the offer import, the documents pipeline and every business rule. The modules `llm`, `scraping`, `text`, `trace_log` and the pure files of `docgen` are pure logic: they access neither the database nor Tauri, so they stay testable in isolation.

```
package.json, vite.config.ts, tsconfig*.json, index.html, eslint.config.js
src/
  views/<view_name>/XxxView.vue      one view = one folder, view-specific components inside
  components/                        components shared across views
  composables/                       reusable view-layer logic (formatting, editable lists)
  stores/app.ts                      global reactive app state
  types/index.ts                     TS DTOs hand-mirrored from src-tauri/src/models
src-tauri/
  src/
    main.rs, lib.rs                  Tauri builder: setup (startup::initialize), managed AppContext, generate_handler!
    error.rs                         AppError, serialized as {kind, message} to the frontend
    models/                          serde DTOs crossing the IPC bridge
    trace_log.rs                     diagnostic traces: stderr, plus a log file configured by the app at startup
    llm/                             Claude client, prompt assembly per pipeline step (templatization, matching, writing, label translation)
    scraping/                        add-by-link flow: URL validation, guarded fetch, typed failures, JSON-LD and page-text extraction, final cleanup
    text/                            deterministic HTML/Markdown to plain text conversion (html_to_plain)
    app_state.rs                     AppState { pool, data_dir, fonts_dir, capture_profile_dir, progress, api_key } and AppContext (state or init error)
    startup/                         .env loading, run mode (DEV/PROD) and folders, log file, pool + migrations, interrupted-generation cleanup, main window
    clock.rs                         single source of the current time and of the stored date formats
    storage_paths.rs                 builds every path of the data folder and resolves paths received from the frontend
    secrets/                         SecretStore trait (swappable in tests) and its system-keyring implementation
    db/                              one file per table + joins.rs, profile.rs, init.rs (pool, migrations), transaction.rs
    services/                        business logic beyond a DB call: applications, generation, settings, API key, Claude target, offer import, sessions
    documents/                       Typst compilation, reference templatization, versions, on-disk document storage, fonts
    docgen/                          deterministic document assembly (cv_data/lm_data builders, labels cache, page fitting, trace) and pure pipeline logic (budgets, linter, selection, period/date formatting, text diff)
    pipeline.rs, progress.rs         generation orchestration, in-memory progress store
    offer_extraction.rs, url_normalize.rs   import cascade, duplicate detection
    commands/                        one #[tauri::command] file per domain, each calling a service or a db function
  tests/integration/                 single integration test binary (main.rs declares one module per file): pure-logic tests (conversion, offer import, docgen), SQLite in-memory tests of the db layer, service-level tests (offer import, generation, lifecycles, end-to-end); common/ shared pure-logic fixtures; shared/ helpers (seed, fake keyring, fake LLM, local offer server, unique temp folders, test AppState)
  tests/fixtures/                    test fixtures (see its README)
  tauri.conf.json, capabilities/, icons/
```

## 4. Frontend/backend boundary (thin-client principle)

Guiding principle: **thin client**. Vue is a rendering and interaction layer; the Rust side is the brain. No business truth is decided in Vue, and the Tauri commands are themselves thin — they call one service or one `db` function and return its result.

- **World state (Rust)**: everything that is *true* about the data (applications, profile, status transitions, timeline rules). This is the business model — it lives in `src-tauri/src/{db,services,documents,pipeline}` and in the pure modules (`llm`, `scraping`, `text`, `docgen`, `models`), nowhere else.
- **View state (Vue)**: what's on screen right now (active tab, open modal, unsaved form content, selected filter). Stays in Vue. Vue's reactivity (`ref`, `computed`, `watch`) manages view state; that is not business logic and is expected.

**Everything the Rust side computes, it sends already computed in the DTO. Vue displays, it never recomputes a business value.**

- Command results are display-ready DTOs with derived fields already included, not raw SQL rows.
- If a rule depends on a setting, the computation lives on the Rust side to stay in a single place.
- Business logic that is more than a single DB call lives in `services/` (or `documents/`), never in a command file.

### Reference example: adding a timeline event

1. The frontend calls the `get_available_events` command (backed by `db/timeline_events.rs`, the sole source of truth for which events are legal in the current status).
2. The user picks an event in `AddEventPopup.vue` and fills in the fields required for that event type.
3. The frontend calls `add_timeline_event`. The Rust side **revalidates** that the event is allowed for the current status (never trusts what the client sent), applies its side effects (creating/updating an interview, changing the status), and writes to `status_history`, all in one write transaction with a single timestamp.
4. If the event is rejected, the frontend surfaces the error without having mutated its local state.

Vue proposes, Rust disposes: Vue never hardcodes the legal transitions, it asks every time via `get_available_events`.

### Type contract (kept manual, not generated)

The source of truth for the IPC contract is the Rust DTOs in `src-tauri/src/models`. `src/types/index.ts` is written by hand, mirroring those DTOs, with no code generation (`tauri-specta` / `ts-rs` not worth the churn yet). Errors reach the frontend as `AppError` serialized `{kind, message}`.

## 5. Data model

The database is a single SQLite file, `findmyjob.db`, in the app data folder (see §12). The Rust side owns the pool (`sqlx::SqlitePool`, `db/init.rs`): WAL journal, foreign keys enabled on every connection, 5 s `busy_timeout`. Every write transaction is opened with `BEGIN IMMEDIATE` (`db/transaction.rs::begin_write`) so a read-then-write transaction takes the write lock up front instead of failing with `SQLITE_BUSY`.

Migrations live in `db/migrations/sqlite/` (`0001_init.sql`, the initial schema; `0002_adresse_libre.sql`, which merges the street and postal code into `adresse` and drops `code_postal`; `0003_adresse_seul_lieu.sql`, which makes the free-text `adresse` the profile's only location field, taking over the former `ville`, and drops `ville`; `0004_contacts_carnet.sql`, the global contact book; `0005_ville_code_postal.sql`, which replaces `adresse` with `ville` and `zip_code`, parsing them from the old free text — FR 5-digit or CH 4-digit postal code with the city after it, or before it when nothing follows; otherwise the whole value as the city unless it looks like a street — then drops `adresse`), embedded in the binary by `sqlx::migrate!` and applied at startup. `db/schema.dbml` is the hand-maintained column-level reference.

Storage conventions:

- **Timestamps** are TEXT, RFC 3339 UTC with fixed millisecond precision (`2026-10-10T08:15:30.125Z`), produced only by `clock.rs`: fixed length, so string order is chronological order (`ORDER BY`, `MAX`). A transaction reads the clock once and passes the same value to all its writes. No column has a time default and there is no trigger: `updated_at` (only on `documents`) is written by the code.
- **Dates** are TEXT `YYYY-MM-DD`; booleans are INTEGER 0/1. Tables are `STRICT`.
- **File paths** stored in the database are relative to the data folder, with `/` as separator, built by `storage_paths.rs`.
- `sessions`, `document_versions` and `candidatures` use `AUTOINCREMENT`: their id names a folder on disk and must never be reused.

Table summary:

| Table | Role |
|---|---|
| `users` | Single user's profile, one row (`id = 1`): name, contact info, driving license (NULL = not given), links, interests |
| `langues` | Languages spoken by the user (`nom` unique), with proficiency `level` |
| `formations` | Degrees and schools in the profile |
| `projets` | Personal projects attached to the profile |
| `experiences` | Work experience entries in the profile |
| `skills` | Skills, grouped by a free-text domain; `(domaine, nom)` unique |
| `settings` | Key-value app settings (`llm_model`, `theme`) |
| `sessions` | A search "session" (name, status) grouping a set of applications, with its own reference CV/cover letter |
| `document_versions` | One row per uploaded reference (CV or letter), never overwritten: files, template, page count |
| `documents` | Pointer `(session_id, doc_type) → current_version_id`, plus `updated_at` |
| `candidatures` | Core entity — one row per application (company, status, metadata, generated document paths, reference versions used) |
| `status_history` | Unified timeline of events for an application (`event_type` and statuses constrained by CHECK) |
| `contacts` | Global recruiter-contact book: name (required), role, email, phone, notes. A contact exists on its own and can be linked to several applications |
| `candidature_contacts` | Link application ↔ contact (composite key, `created_at` gives the link order, CASCADE on both sides: deleting an application removes its links, never the contacts) |
| `interviews` | Scheduled or completed interviews for an application |
| `interview_contacts` | Interviewers of an interview (composite key, CASCADE on both sides); only contacts linked to the interview's application are accepted (`db::timeline_events::plan_interview`), and unlinking a contact from an application also removes it from that application's interviews |

Deleting a session cascades to everything that depends on it in the database, and `services::sessions::delete` also removes its `documents/sessions/<id>/` folder. Duplicate skills and languages are checked before writing, inside the transaction, and refused with a French message (never the raw SQLite constraint error). Skills and languages are sorted case-insensitively (`COLLATE NOCASE`).

## 6. Application lifecycle

### Statuses (11 values, `candidatures.status`)

`a_postuler`, `postule`, `relance`, `entretien`, `entretien_passe`, `offre`, `refus`, `abandon`, `ghosting`, `accepte`, `offre_refusee`.

`entretien_passe` distinguishes an upcoming interview from one already completed but with no response from the company yet. `offre` is an intermediate status: receiving an offer is not terminal, it can still be accepted (`accepte`) or declined (`offre_refusee`). The negative terminal statuses (`abandon`, `refus`, `ghosting`, `offre_refusee`) move the application to the trash view on the frontend (`TRASHED_STATUSES` in `src/composables/candidature_display.ts`); `accepte` is terminal but stays visible since it's a positive outcome.

### Timeline event types (15 values, `src-tauri/src/models/event_type.rs`)

`creation`, `candidature_envoyee`, `relance`, `entretien_planifie`, `entretien_passe`, `offre_recue`, `refus_recu`, `ghosting`, `desistement`, `accepter_offre`, `decliner_offre`, `message_envoye`, `message_recu`, `appel`, `note`.

Each event has an optional effect on the status (`EventType::changes_status`), e.g. `entretien_planifie` → `entretien`, `accepter_offre` → `accepte`, while `message_envoye`, `appel`, or `note` never change the status. `creation` and `candidature_envoyee` are never offered by `get_available_events` — they're driven by the existing creation and "apply" flows instead. `EventType` is the source of validation in Rust; the column also carries a `CHECK` listing the same 15 values.

### Which events are offered for a given status

`src-tauri/src/db/timeline_events.rs::available_events` is the sole source of truth. A non-terminal application (`a_postuler`, `postule`, `relance`, `entretien`, `entretien_passe`, `offre`) can always receive `message_envoye`, `message_recu`, `appel`, `note`, `desistement`; the remaining events are only offered for a specific subset of statuses (e.g. `relance` from `postule`/`relance`/`entretien_passe`, `accepter_offre`/`decliner_offre` only from `offre`).

### The "apply" flow

Three commands cover the transition from `a_postuler` to `postule` (`services/candidatures.rs`). The LLM call, when there is one, happens outside any transaction; only the status change opens one. The frontend then exports the documents to the downloads folder.

- `postuler_sans_message` — marks the application applied.
- `postuler_par_mail` — has the AI draft an application email (`llm::candidature_message::generate_application_email`).
- `postuler_par_message` — has the AI draft a short application message.

All three then record the `candidature_envoyee` event and refuse (`AppError::Conflict`) if the application is no longer in the `a_postuler` status.

## 7. Documents pipeline (CV / cover letter tailoring)

Each session owns one reference CV and one reference cover letter, stored under `documents/sessions/<session_id>/{cv,lettre_motivation}/` in the app data folder. The user's reference `.typ` is never modified. **The reference is the source of truth for structure and text**: nothing is ever removed from it, and a text with no field in the schema (a mention, a sentence, an address line) stays as is in every generated document. **The profile is the source of truth for every piece of information it holds**: when the reference shows an information the profile knows (name, city, school, company…), the profile's value replaces it, even inside a sentence; when the profile is empty on that information, the reference's value is kept. Generation adapts the prose to the offer from the profile, without inventing anything.

### 7.1 Templatization, at upload

- **Saving a reference** (`documents/storage.rs::save_reference`) compiles the `.typ`, then converts it into a template (`documents/templatize.rs::templatize`, one Claude call with structured output, `llm::templatize`). Both happen in memory, before any write and outside any transaction: a failure writes nothing. Then, in one write transaction, the version row is inserted (its id names the folder), the files are written under `<doc_type>/versions/<version_id>/` as `reference.{typ,pdf}`, `template.typ` and `example_data.json`, and the `documents` pointer moves to the new version; any failure rolls the transaction back and removes the folder. Versions can be restored (`documents/reference_versions.rs`).
- **`template.typ`** is the same document, where every value the schema describes is read from a `data.json` file following a canonical schema (`models::doc_data`: `CvData`, `LmData`, identical for every user): candidate information, recipient and date of a letter, prose written per offer, and `labels` (section titles, formulas — translatable). Every other text of the reference is copied word for word into the template, and a schema value inside a copied sentence is inserted from `data`. The recipient's other lines (department, postal address) are `recipient.extra_lines`, empty at generation since the offer does not give them. Lists are rendered with loops, an empty optional field hides its whole line without orphan separators, values are inserted as strings (never evaluated as markup), and the letter date comes from the data.
- **`example_data.json`** is the reference's content in that schema, plus `detected_lang`, for a CV `period_style` (month or year per section), and `guidance` (`models::prose_guidance`): the reference's general instructions and, for each prose field (`identity.title`, `summary`; `subject`, `paragraphs[i]`), its split into zones tagged `[FIXE]`, `[VARIABLE]` or `[ADAPTABLE]` with their instruction, as written in the reference's comments. It is the style model, the source of the length budgets, and the fallback for any information the profile lacks.
- **Validation** (`documents/templatize.rs`): the template must load `data.json` and must not use `eval`, `datetime.today` or `read`; rendered with `example_data`, it must not exceed the reference's page count and must render exactly the words of the reference (compared without leading/trailing punctuation; only a date computed by the reference may differ); rendered with empty data, it must compile; rendered with only the secondary fields empty, it must leave no orphan separator (`documents/sparse_check.rs`: each field is then emptied alone so the feedback names the field whose separator to condition; if orphans are the only problem left at the last attempt, the template is accepted with a logged warning); rendered with a unique marker in every field and one item per list (sentinel render, `docgen::sentinel_data`), its remaining hard-coded text must not reproduce any value that has a field (`docgen::example_facts`: a name, city, company or summary left in the template would never be replaced by the profile's). The zones of `guidance` must rebuild each prose field exactly (`docgen::guidance_consistency`). The sender's location is `identity.ville` + `identity.zip_code`, composed by the template in the reference's order and punctuation; a letter's date line uses the city only. Failures are sent back as feedback (compiler errors with the offending line, missing/extra words), 2 retries at most, then the upload fails with the last problems; a retry that reproduces exactly the problems of an earlier attempt gets an extra line telling the model it is oscillating. A reference converted by an earlier version (template that still reads a removed field such as `identity.adresse`, or `example_data.json` without `guidance`) is refused at generation with `AppError::OutdatedReference` (re-import the reference). `page_count` is the page count of the template rendered with its example data.
- A reference is **ready** (`references_ready`, `DocumentsStatus`) once its template exists.

### 7.2 Generation, per application

`src-tauri/src/pipeline.rs::generate_documents`, run as a background task in Tauri's async runtime by `services/generation.rs` once `create_candidature` has inserted the application. The progress entry (step 0) is published before the command returns, so the first poll never misses it; progress lives in memory (`progress.rs`). If generation fails, the application and its partial files are deleted. Closing the app kills the task: at the next startup, every `a_postuler` application with no document at all is treated as an interrupted generation and deleted with its files.

1. Offer analysis (`llm::offer::analyze_offer`); its `language` is the documents' language.
2. Matching (`llm::matching::match_profile`), then selection (`docgen::selection::select`): every experience is kept, plus the N most relevant projects (N = number of projects in the reference). `highlights` / `keywords_covered` / `match_type` steer what to say; they are not text to copy.
3. Budgets (`docgen::budgets`) from `example_data`: exact element counts (paragraphs, projects, skill groups, bullets per entry by rank), word counts at ±10 %, page count. Words of `[FIXE]` zones are never reduced when the budgets shrink.
4. Labels in the offer's language: the reference's own labels, or a Claude translation cached per reference version and language (`docgen/labels_cache.rs`, `labels.<lang>.{cv,lm}.json`).
5. Writing, CV and letter in parallel (`tokio::try_join!`): `llm::cv_writing::write_cv` (title, summary, bullets per selected entry id, skill groups) and `llm::lm_writing::write_lm` (subject, paragraphs). Prompts carry the offer analysis, the selected profile facts, the reference's prose clearly delimited as a style model, the reference's instructions (`writing_rules::guidance_block`: general instructions and tagged zones, which take precedence over the style rules), the budgets and the target language. A `[FIXE]` zone is copied word for word, only the profile's information updated in it; `[VARIABLE]` comes from the offer; `[ADAPTABLE]` and untagged prose are written from the profile. Each output is reviewed (`llm::writing_loop`): blocking validation (ids match the selection, every skill exists in the profile, no empty field) plus `llm::prose_checks`: the linter (`docgen::lint`: em/en dashes, Markdown bold, backticks and forbidden phrases from `docgen::forbidden_phrases`, each allowed when the reference uses it; budgets), run outside the text of fixed zones, and the fixed-zone check (`docgen::fixed_zones`: a word of a fixed zone may be missing only if it belongs to an information of the reference). Violations trigger a new call with the previous output and the problem list, 2 retries at most; after the last one, remaining dashes are replaced mechanically outside fixed zones and other violations are logged without blocking.
6. Deterministic assembly (`docgen/build_cv_data.rs`, `docgen/build_lm_data.rs`): identity, titles, companies, periods (`docgen::period_format`, in the reference's style), education, languages, interests and recipient (company and location of the application, `docgen/recipient.rs` — the letter never mentions a contact) come from Rust; prose comes from step 5. Fallback to the reference (`docgen/document_fallback.rs`): an identity field the profile leaves empty takes the `example_data` value, and a list the profile has no entry for (experiences, projects, education, skills, languages, interests) takes the reference's entries; the recipient and the date come from the application only. The professional subtitle under the name (`Identity.title`, shared by both documents) is the CV's written `title`; the letter, assembled after the CV, reuses the fitted CV's value. `Recipient.contact_name` / `contact_role` are always empty: they are only kept so that templates converted before this rule still compile, and are absent from the templatization schema.
7. Compilation of `template.typ` with the data (`docgen/page_fit.rs`): if the PDF exceeds the reference's page count, writing is re-run with budgets reduced by 15 % (cumulative), twice at most, then the document is kept with a warning.

No LLM produces Typst during generation. **Compilation** uses `typst-as-lib` (`documents/typst.rs`) inside `spawn_blocking`, serving the data as a virtual `data.json` file, with fonts from the system, `typst-kit` embedded fonts and fonts imported by the user into `<data folder>/polices/<family>/`.

**Artifacts**: `.../artefact/<candidature_id>.pdf` and `.data.json` (the template version is `candidatures.{cv,lm}_reference_version_id`). **Trace**: `documents/sessions/<session_id>/debug/<candidature_id>/` holds `offer_analysis.json`, `matching.json`, `selection.json`, `budgets.json`, `cv_content.json`, `lm_content.json`, `lint_report.json` and `attempts.json`.

## 8. AI layer

### HTTP client (`src-tauri/src/llm/client.rs`)

Raw HTTP calls to Anthropic's Messages API (`https://api.anthropic.com/v1/messages`, header `anthropic-version: 2023-06-01`) via `reqwest` — there is no official Rust SDK for the Anthropic API. `llm` never reads the key itself: every `llm` function receives a `ClaudeTarget { api_key, model, caller }` (`llm::target`) from its caller. `caller` is an `Arc<dyn LlmCaller>` (`llm::caller`): the app always uses `AnthropicCaller`, the HTTP implementation in `client.rs` (held by `AppState.llm`), and tests substitute canned answers. The provider stays Anthropic-only; there is no configurable base URL. `ApiKey` (`llm::api_key`) has no `Display` nor `Serialize` and a masked `Debug`, so the key cannot leak into a trace or an error message.

### API key (`services/api_key.rs`, `services/claude_target.rs`, `secrets/`)

`ApiKeyAccess::resolve` is the only place that decides where the key comes from, from the run mode decided at startup:

| Mode | Source | Ignored |
|---|---|---|
| DEV | `ANTHROPIC_API_KEY` from the loaded `.env` | system keyring (never opened) |
| PROD | system keyring, service = Tauri identifier (`com.findmyjob.app`), account = `anthropic_api_key` | any `ANTHROPIC_API_KEY` in a `.env` (never read, never copied to the environment) |

Every code path that calls Claude goes through `ClaudeTargetSource::resolve` (configured model + key; the keyring call runs in `spawn_blocking`). The offer import resolves it only when it reaches the LLM stage, so a JSON-LD import still works without a key. The keyring sits behind the `SecretStore` trait (`secrets/store.rs`); `secrets/keyring_store.rs` implements it with `keyring::Entry` and turns an unavailable keyring (no Secret Service, locked store, unsupported platform) into an explicit French `AppError::Config`. There is no fallback to a file.

Commands: `get_api_key_status` (present, source `env`/`keyring`, last 4 characters; the full key never crosses IPC), `save_api_key` (trimmed, refused if empty) and `delete_api_key`. The last two are PROD only; in DEV they return a `Validation` error pointing to the `.env`. When the key is missing, AI features fail with « Aucune clé API Anthropic configurée. » followed by « Saisis-la dans les Paramètres. » (PROD) or « Renseigne ANTHROPIC_API_KEY dans le fichier .env. » (DEV).

- `thinking` is explicitly disabled on every request: the pipeline is a single-turn structured generation, and extended thinking shares the same output token budget as the visible response — leaving it on by default truncated responses before any useful content.
- Structured outputs (JSON Schema) are used wherever a schema is provided, to guarantee a strictly conformant response (offer analysis, matching, templatization, CV/LM writing, label translation). The API caps the compiled grammar size: the templatization schema for a CV sits close to that cap, so fixed-value fields are plain strings checked in Rust rather than `enum`s.
- `MAX_TOKENS = 8192` by default; templatization (full template + example data in one response) uses 16384.
- One shared `reqwest` client with a 300 s request timeout and a 10 s connect timeout (long Typst/prose generations can take minutes).
- Two explicit failure cases are turned into a clear `AppError::Llm` rather than a downstream JSON-parsing error: `stop_reason == "refusal"` (safety classifier) and `stop_reason == "max_tokens"` (truncated response).

### Application message drafting (`llm::candidature_message`)

Two functions (`generate_application_email`, `generate_application_message`), plain text ready to paste, one Claude call each, no structured output. Shared rules: only rephrase what already exists in the profile, no placeholders, no markdown, minimal signature (first + last name only). When a contact is linked to the application, the first one (link order) is passed with its role and the greeting becomes « Bonjour <name as written> » (or its direct equivalent in the offer's language), never with a guessed title; without a contact, the email keeps « Madame, Monsieur » and the message has no greeting.

## 9. Tauri commands (`src-tauri/src/commands/`)

79 commands are registered in `lib.rs` (`generate_handler!`). Each takes the managed `AppContext` and calls one service or `db` function; if startup failed, every command returns that error (`AppError::Config`), which `App.vue` shows on its startup error screen.

| Module | Commands (summary) |
|---|---|
| `users` | user existence check, create, get, aggregated profile, update description/info |
| `experiences` / `formations` / `langues` / `projets` / `skills` | add / update / delete per sub-entity |
| `sessions` | list / create / update / delete (with its documents folder) |
| `candidatures` | list by session, create (+ background generation), generation progress, get, update, delete (à postuler only), apply (no message / by mail / by message), find by normalized offer URL |
| `contacts` | contact book, contacts of an application, create (optionally linked to an application), update, delete (all its links go with it), link to / unlink from an application |
| `status_history` | history for an application |
| `timeline_events` | list available events, add an event |
| `interviews` | pending interviews for an application, all interviews for a session |
| `scraping`, `capture` | offer extraction from a URL, from the capture window, or from pasted text; see « Offer import by link » below |
| `documents`, `document_versions` | reference CV & cover letter (get, import, compile), status, versions (list, preview, restore), attached PDFs (info, preview, export), downloads folder, text export |
| `fonts` | list / import / delete an imported font family |
| `settings` | LLM model, theme, connection test |
| `system` | open a URL with the OS |

Paths received from the frontend (`get_document_info`, `get_document_pdf`, `download_document`) are relative paths resolved under the data folder by `storage_paths::resolve_untrusted`: absolute paths and `..` are refused, and an existing file is canonicalized before the containment check.

### Offer import by link

An offer enters the app only when the user adds it, one page at a time: there is no crawl, no automatic discovery and no retry loop against a site that refuses. Nothing bypasses a site's protections (no proxy, no identity rotation, no captcha solving).

**Fetch** (`scraping::fetch`, used by `scrape_offer`): the URL is validated first (`url_validation.rs`: `http`/`https` only, host required, no credentials, no `localhost`, single-label or `.local` names, and no loopback/private/link-local/CGNAT IP). A fresh client per fetch sends browser `Accept`/`Accept-Language` headers, keeps cookies only for the redirect chain, follows at most 10 redirects (each target revalidated) and resolves DNS through `safe_resolver.rs`, which drops non-public addresses. The body is read in chunks and capped at 5 MB; a non-HTML `Content-Type` or an empty page is refused.

**Typed failures** (`scraping::failure::ScrapingFailure`): `invalid_url`, `offer_unavailable` (404/410), `access_denied` (401/403/429/999), `timeout`, `network`, `not_html`, `empty_page`, `too_large`, `insufficient_content`. They travel as `AppError::Scraping(code)`, serialized `{"kind":"Scraping","message":"access_denied"}`; the technical detail goes to the log, and the UI picks the message and the next step.

**Cascade** (`src-tauri/src/offer_extraction.rs::run_cascade`, shared by `scrape_offer` and `capture_offer_page`), on the raw HTML:

1. **JSON-LD, no LLM** (`scraping::json_ld`): every `application/ld+json` block is read, and a `JobPosting` is searched in objects, arrays and `@graph` (`@type` string or array). Mapping: `title` → poste, `hiringOrganization.name` → entreprise, `jobLocation` (object or array) → localisation, prefixed by « Télétravail » when `jobLocationType` is `TELECOMMUTE`, `baseSalary` → readable text (`salary_format.rs`, French number formats, empty or zero amounts ignored), `description` → description, `applicationContact` (`ContactPoint` or `Person`, object or array; `name`, `jobTitle` or `contactType`, `email`, `telephone`) → contact. Contract type (`contract_type.rs`): `employmentType: INTERN` → `stage`, otherwise an explicit keyword in the title (stage, stagiaire, intern, alternance, apprentissage, CDI, CDD, intérim), otherwise `null` (a title naming two different types also gives `null`). The stage succeeds when the title is present and the cleaned description exceeds 200 characters.
2. **Page text + LLM** (`scraping::extract::extract_page_text`, then `llm::scraping_extract`): among the semantic containers (`article`, `main`, `section`, `[class*=job]`…), the one with the longest text wins (fallback `<body>`); navigation, headers, footers and forms are dropped; headings and lists are kept; truncation (12 000 characters) happens after isolation. `<title>` and `og:title` are passed to the LLM as hints. The prompt asks for the description copied word for word, with the page's own section titles, and for a `contact` only when the offer explicitly names a person (`null` otherwise, nothing invented). The stage fails with `insufficient_content` when the text is too short or looks like a JavaScript shell, an anti-bot challenge or a lone cookie banner (`content_check.rs`), or when the LLM returns an empty title or description.

**Final cleanup** (`scraping::finalize::finalize_fields`), on every path (JSON-LD, LLM, pasted text): the description goes through `text::html_to_plain` (no tags, emoji or Markdown decoration; headings and lists kept; escaped HTML from JSON-LD decoded), short fields are reduced to one line, a contract type outside the form's values is dropped, and a contact without a name is dropped. The extracted contact only prefills a row of the form's contacts section (editable, removable); nothing is saved before the form is submitted, and `create_candidature` inserts the application and its contacts (new ones or picked from the book) in one transaction. The commands return `OfferExtraction { fields, stage }` with `stage` = `json_ld` or `llm`.

**Capture window** (`src-tauri/src/commands/capture.rs`): when the app cannot read the page by itself (access refused, content rendered by JavaScript), the user opens it in a secondary `capture` window, accepts cookies or logs in, then clicks « Capturer cette page » in the main window. The rendered HTML is read on the Rust side with `WebviewWindow::eval_with_callback` (no IPC from the page), capped at 5 MB, and passed to `services::scraping::extract_from_html`, so it goes through the same cascade. The window belongs to no capability: Tauri rejects every IPC call from a remote origin without a `remote` capability, so the remote page cannot reach any app command. The URL is validated before loading (a local URL such as the dev server would be treated as an app page with IPC access). The window's WebView profile (`capture_webview/`, in the app local folder in PROD or in `dev-data/` in DEV — see §10) keeps the site session between uses, isolated from the main window.

**Duplicates**: `find_candidatures_by_url` returns existing applications, across all sessions, whose `url_offre` matches after normalization (`src-tauri/src/url_normalize.rs`: no scheme, `www.`, query, fragment or trailing slash, lowercase). The screen shows them as a non-blocking warning.

## 10. Startup and diagnostics

`startup::initialize` runs once in Tauri's `setup`, before any command:

1. Loads `.env` (see §8) and keeps the path of the file loaded.
2. Decides the run mode with `startup::run_mode::resolve` — a pure function of the `.env` path (or its absence), the value of `MODE` and the Tauri folders. `MODE` is read from the `.env` that was loaded: no `.env`, `MODE` absent or `MODE=PROD` → PROD; `MODE=DEV` → DEV; any other value is a startup error (case is ignored). An invalid value names the value received and the two accepted values; it becomes the initialization error shown by the UI.

   | Mode | Data (`findmyjob.db`, `documents/`, `polices/`) | Log file | Main window WebView profile | Capture window profile |
   |---|---|---|---|---|
   | PROD | `app_data_dir` (Windows `%APPDATA%\com.findmyjob.app\`; macOS `~/Library/Application Support/com.findmyjob.app/`; Linux `~/.local/share/com.findmyjob.app/`) | `<app log folder>/findmyjob.log` (Windows `%LOCALAPPDATA%\com.findmyjob.app\logs\`) | Tauri default (Windows `%LOCALAPPDATA%\com.findmyjob.app\EBWebView`) | `<app local data>/capture_webview` |
   | DEV | `<folder of the .env>/dev-data/` | `dev-data/logs/findmyjob.log` | `dev-data/webview/` | `dev-data/capture_webview/` |

   In DEV nothing is written to the system app folders. The rest of the code receives these folders (`AppState.data_dir`, `AppState.capture_profile_dir`) without knowing the mode.
3. Creates the data and log folders (so `dev-data/` when missing), points `trace_log` at `findmyjob.log` and logs the mode and folders. Every trace (fetch failures, templatization retries, writing-loop feedback, page fitting, cleanup failures) goes to stderr and to that file — a release build on Windows has no console. Traces emitted before this step (the `.env` lookup) reach stderr only.
4. Creates `polices/`, opens the pool and applies the migrations.
5. Deletes interrupted generations (see §7.2).

The result is managed as `AppContext`: either the `AppState`, or the initialization error returned by every command. Then `setup` creates the main window (`startup/main_window.rs`): it is declared with `"create": false` in `tauri.conf.json` so that its WebView profile can follow the mode.

## 11. Features — implemented, partial, and planned

### Implemented end-to-end

- Onboarding and full profile management (experiences, formations, languages, projects, skills).
- Sessions: CRUD, reference CV/cover letter per session (import, compile, templatization, version history with restore, PDF preview).
- Applications: creation with mandatory AI-generated documents (failure blocks creation), table view per session, editable detail while `a_postuler`, typed timeline, interview management, trash for negative terminal statuses.
- Recruiter contacts: global book shared across applications, picked or created from the new-application form and the application detail, editable and removable whatever the status, several interviewers per interview (shown in the planning), used by the application email/message (never by the cover letter).
- Apply flow: no message / by mail / by message, all exporting documents to disk.
- Offer import from a link (JSON-LD, then page text + LLM), from a page captured in the FindMyJob window, or from pasted text, prefilling the new-application form; choice screen on failure, non-blocking duplicate warning.
- Interview planning (week/month views).
- Settings: LLM model selection, Anthropic API key (system keyring in PROD with save/delete, read-only `.env` state in DEV), connection test (run again after a key is saved), fonts, theme.
- CI/CD: lint, clippy and tests (Linux + Windows) on every PR to `dev`, full Tauri build check on PRs to `staging`/`main`, Conventional Commits, `dev → staging → main` branch flow, semantic-release on `main`.

### Planned / out of MVP scope

- A dedicated contact-book page (the book is reachable today only through the application forms).
- Smart follow-ups: automatic detection of stale applications and a drafted follow-up message (distinct from the existing manual `relance` status/event).
- A standalone profile-to-offer matching feature (project ranking by relevance), distinct from the internal matching step already used inside the tailoring pipeline (`llm::matching::match_profile`).
- A provider-agnostic LLM layer (OpenAI-compatible, configurable base URL) to support Ollama Cloud or a local runtime — not implemented; the provider is Anthropic-specific today (see `CLAUDE.md`).
- Multi-device sync, mobile client, mail/calendar integrations.

## 12. Repository, CI/CD, and conventions

### Commit convention

Conventional Commits drive automated versioning: `<type>(<scope>): <description>`. `feat:` bumps minor, `fix:` bumps patch, `feat!:`/`fix!:`/a `BREAKING CHANGE` footer bumps major. `docs:`, `style:`, `refactor:`, `test:`, `chore:` don't trigger a release. Full detail in `docs/commit.md`; pull request conventions in `docs/pull_request.md`.

### Branch flow and CI (as actually configured)

- `feature/*` → PR to `dev` — `.github/workflows/ci.yml`: `frontend` job (`npm run lint` + `npm run build`, from the repository root), `rust` job (`cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace`, matrix `ubuntu-22.04` + `windows-latest`, `fail-fast: false`, Cargo cache per OS; the desktop crate compiles Typst and a bundled SQLite; `CARGO_INCREMENTAL=0` and no debug info (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`) to keep the Linux runner's disk from filling up, with `df -h /` logged before and after the tests on Linux). The Rust toolchain is pinned by `rust-toolchain.toml` (exact stable version, clippy, rustfmt, macOS targets) and installed with `rustup toolchain install`.
- PR to `staging` or `main` — `.github/workflows/build-check.yml`: full Tauri build (`npm run tauri build`) on the same matrix and with the same setup as `release.yml`, without version injection, publication or upload.
- `dev` → PR to `staging`, `staging` → PR to `main` — `.github/workflows/gatekeeper.yml` blocks any other source branch (only `staging` can merge into `main`, only `dev` can merge into `staging`).
- Push to `main` — `.github/workflows/release.yml`: `semantic-release` (`.releaserc.json`) determines the version, then a Tauri build matrix (macOS ARM64/x64, Ubuntu 22.04, Windows) and a GitHub release if a version was computed.

### Running locally

Everything runs on the user's machine, in one process; there is no server and no Docker.

- `npm install && npm run tauri dev`, from the repository root. Tests: `cargo test --workspace`, from the repository root. In development, `.env` is found at the repository root by walking up from the current directory.
- With `MODE=DEV` in that `.env`, all data goes to `dev-data/` at the repository root (gitignored); without it, the app uses the system folders like an installed app (PROD). See §10 for the full table.
- PROD data folder (Tauri `app_data_dir`, identifier `com.findmyjob.app`): `findmyjob.db`, `documents/`, `polices/`. On Windows, `%APPDATA%\com.findmyjob.app\`; on macOS, `~/Library/Application Support/com.findmyjob.app/`; on Linux, `~/.local/share/com.findmyjob.app/`. An installed app reads its `.env` from the app config folder (the same folder on Windows).
- API key: in DEV, `ANTHROPIC_API_KEY` in the `.env`; in PROD, entered in Paramètres and stored in the system keyring (see §8). On Linux, PROD needs a running Secret Service (GNOME Keyring, KWallet).

## 13. Known gaps and points of attention

- CI's `rust` job runs `cargo test --workspace` on every PR. Coverage: HTML/Markdown-to-text conversion, the offer import's pure logic (synthetic HTML, no network), the documents pipeline's pure logic (including template compilation on a fixture), the whole `db` layer on an in-memory SQLite database (CRUD, cascades, constraints, duplicates, timestamp ordering, one timestamp per transaction) and the file-handling services (path resolution, interrupted-generation cleanup, session folder removal), and the API key rules on an in-memory fake keyring (source per mode, missing key, save/delete, DEV refusal, masking); no test touches the real keyring. Service-level tests, at the level the Tauri commands call, add the following:
  - offer import by link and by capture: real reduced pages served by a local `wiremock` server (127.0.0.1, random port), finalized fields, typed failures;
  - CV/LM generation: profile-sourced `data.json`, real Typst compilation, one-page CV, prose lint, empty-prose retries, failure cleanup;
  - profile, session and application-deletion lifecycles;
  - one end-to-end scenario.

  The crate builds a single integration test binary from `src-tauri/tests/integration/` (`main.rs` declares one module per test file), which keeps disk use and link time down; every new integration test is a module of `tests/integration/`, never a new file at the root of `tests/`. Since all integration tests share one process, each test gets its own temporary folder (`tests/integration/shared/temp_dir.rs`). The Claude API is replaced by `tests/integration/shared/fake_llm.rs`, an `LlmCaller` with canned answers. Loopback fetching (`fetch_html_allowing_loopback`) is compiled only under the `test-support` feature, which only the crate's dev-dependency on itself (`findmyjob = { path = ".", features = ["test-support"] }`) enables, so a plain `cargo test --workspace` has it and no app build does. No test reaches the network, the Claude API, `.env` or `dev-data`. The Tauri commands themselves and the capture window remain untested.
- Generation progress is in memory only: closing the app during a generation loses it, and the half-created application is removed at the next startup (there is no way to resume a generation).
- Only the profile was carried over from the PostgreSQL version: sessions, references and applications were not migrated.
