# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

FindMyJob is an open-source desktop app (Tauri 2) that helps students track job applications (internships, apprenticeships, permanent contracts). It's a standalone application — one process, no server, no Docker — laid out as a standard Tauri project: the Vue frontend at the repository root and a single Rust crate in `src-tauri/`, which owns the SQLite database, the AI pipelines and every business rule. Offers enter the app only through their link (or pasted text) — there is no automatic offer collection. See [`docs/project.md`](docs/project.md) for the exhaustive technical reference — this file is the day-to-day working guide.

## Project layout

```
src/        Vue 3 frontend (package.json, vite.config.ts, tsconfig*.json at the repository root)
src-tauri/  Tauri backend, crate `findmyjob` (lib `findmyjob_lib`): SQLite pool, LLM calls, offer extraction by link, the documents pipeline (Typst generation) and every business rule.
```

The root `Cargo.toml` is a workspace with the single member `src-tauri` (`Cargo.lock` and `target/` stay at the root). The modules `llm`, `scraping`, `text`, `trace_log` and the pure files of `docgen` access neither the database nor Tauri.

The Rust side (`src-tauri/`) is the single source of truth for business rules (status transitions, timeline rules, validation). The Vue frontend never reimplements a rule — it calls a Tauri command and displays what comes back. Tauri commands are themselves thin: they call one service or one `db` function.

## Where things live

| What | Where |
|---|---|
| Shared DTOs (serde structs crossing the IPC bridge) | `src-tauri/src/models/*.rs`, one file per domain |
| Claude API client and prompt assembly | `src-tauri/src/llm/*.rs` (`client.rs` is the raw HTTP client; every call takes a `ClaudeTarget` = key + model + caller, `llm` never reads the key itself; the caller is the `LlmCaller` trait in `caller.rs`, always `AnthropicCaller` in the app, held by `AppState.llm`) |
| Anthropic API key (DEV `.env` / PROD system keyring) | `src-tauri/src/services/api_key.rs` (`ApiKeyAccess::resolve`, the only place that picks the source), `services/claude_target.rs` (key + model for every Claude call), `secrets/` (`SecretStore` trait, keyring implementation) |
| Diagnostic traces (stderr + log file) | `src-tauri/src/trace_log.rs` — use `trace_log::log`, never `eprintln!` |
| Database access (SQLite via sqlx) | `src-tauri/src/db/*.rs`, one file per table; `init.rs` (pool, migrations), `transaction.rs` (`begin_write`) |
| Business logic beyond one DB call | `src-tauri/src/services/*.rs` (applications, background generation, settings, offer import, sessions) |
| Session statistics (funnel, outcomes, weekly activity, what works, durations, follow-ups) | `src-tauri/src/db/session_stats.rs` (raw loading), `src-tauri/src/services/session_stats/` (pure computation, `now`/`today` as parameters; `load.rs` reads the clock), DTO `models/session_stats.rs`, command `get_session_stats`; display only in `src/views/tracking/components/stats/` |
| Current time and stored date formats | `src-tauri/src/clock.rs` — the only place that reads the clock |
| Paths of the data folder | `src-tauri/src/storage_paths.rs` — the only place that builds or resolves them |
| Startup (`.env`, run mode, log file, folders, pool, cleanup, main window) | `src-tauri/src/startup/` — the DEV/PROD decision is `run_mode::resolve`, the only place that picks the data, log and WebView folders |
| Offer import by link (URL validation, guarded fetch, JSON-LD then page text + LLM, final cleanup) | `src-tauri/src/scraping/` (pure logic), `src-tauri/src/llm/scraping_extract.rs`, cascade in `src-tauri/src/offer_extraction.rs`, entry points in `services/scraping.rs` |
| Capture window (user-driven page capture when the app can't read a page) | `src-tauri/src/commands/capture.rs` — captures and passes the HTML to `services::scraping::extract_from_html`, no extraction logic |
| Documents pipeline (CV/LM tailoring + Typst) | `src-tauri/src/pipeline.rs` (orchestration), `docgen/` (pure logic only: data assembly, budgets, linter, selection, translation checks, sentinel data), `documents/` (reference markup, two-step templatization and its validation, reference loading, translation caches, page fitting, generation trace, storage, versions, compilation, fonts) |
| Reference markup format (`[CONSIGNES]`, `[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`) | `docs/reference-markup.md` (spec), `src-tauri/src/documents/markup/` (deterministic parsing with `typst-syntax`) |
| Supported document languages and their names | `src-tauri/src/models/doc_lang.rs` — the only place |
| Detailed context of experiences/projects (prompt DTO, letter block, copy check) | `src-tauri/src/llm/profile_prompt.rs` (profile JSON for matching and application messages), `src-tauri/src/llm/detailed_context_block.rs` (letter only), `src-tauri/src/docgen/verbatim_check.rs`; length limit in `db/common.rs::normalize_contexte` |
| Claude call traces (model, tokens, duration, request id, prompts) | `src-tauri/src/llm/call_record.rs`; generation `debug/<cid>/llm_calls.json`, DEV `templatize_debug/` |
| Tauri commands | `src-tauri/src/commands/*.rs`, one file per domain, registered in `lib.rs` (`generate_handler!`) |
| TypeScript types | `src/types/index.ts` (hand-mirrored from `src-tauri/src/models`, no codegen) |
| Interface language (FR / EN) | `src/i18n.ts` (`vue-i18n` instance, `t` for non-component code, `dateLocale()`), `src/locales/fr.ts` (reference) and `en.ts`, `src/composables/useUiLanguage.ts` (setting `ui_lang`), `src/composables/backend_error_message.ts` (backend messages shown in English) |
| Vue views and components | `src/views/<view_name>/` (one folder per view) and `src/components/` (shared) |

## Code conventions

### Rust (`src-tauri/`)

| Aspect | Rule |
|---|---|
| Documentation | `///` for public items, `//!` for modules |
| Linting | `cargo clippy --workspace -- -D warnings` must pass with zero warnings |
| Imports | Grouped by category, separated by a blank line: std → external crates → internal modules |
| Modules | One file per responsibility; `mod.rs` files contain only `pub mod`/`pub use`, never logic |

### TypeScript/Vue (`src/`)

| Aspect | Rule |
|---|---|
| Documentation | JSDoc for exported functions and composables |
| Linting | `npm run lint` (run from the repository root) must pass with zero warnings |
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
2. **DB module** — add/extend the table's file in `src-tauri/src/db/`, with tests in `src-tauri/tests/integration/` (in-memory SQLite, `tests/integration/shared/seed.rs`).
3. **Service** — if the logic is more than one DB call (rules, orchestration, files), put it in `src-tauri/src/services/`.
4. **Tauri command** — add the command in `src-tauri/src/commands/<resource>.rs`, calling the service or db function via `ctx.state()?`; register it in the `generate_handler!` list in `src-tauri/src/lib.rs`.
5. **TypeScript type** — mirror the DTO by hand in `src/types/index.ts`.
6. **Vue component** — wire the `invoke()` call into the relevant view or component.

Skipping a step usually means the feature is backend-only — that's a valid intermediate state, but call it out rather than letting a function silently go unused.

## Running the project locally

```bash
cp .env.example .env      # MODE=DEV for a local dev-data/ folder, ANTHROPIC_API_KEY (read in DEV only)
npm install
npm run tauri dev
```

There is no server and no Docker. At startup the app loads `.env` (app config folder first, then the current directory and its parents — in dev, the repository root), decides the run mode, creates its data folder, opens `findmyjob.db` and applies the migrations.

Run mode (`startup/run_mode.rs`): `MODE` is read from the `.env` that was loaded: no `.env`, `MODE` absent or `MODE=PROD` → PROD; `MODE=DEV` → DEV; any other value is a startup error (case is ignored).

- **PROD** — data folder = Tauri `app_data_dir` (identifier `com.findmyjob.app`; Windows `%APPDATA%\com.findmyjob.app\`: `findmyjob.db`, `documents/`, `polices/`); log file `<app log folder>/findmyjob.log` (Windows `%LOCALAPPDATA%\com.findmyjob.app\logs\`); WebView profiles in the app local folder (`EBWebView/`, `capture_webview/`).
- **DEV** — everything in `dev-data/` next to the `.env` that declared `MODE=DEV` (the repository root): `findmyjob.db`, `documents/`, `polices/`, `logs/findmyjob.log`, `webview/` (main window profile), `capture_webview/`. Nothing goes to the system app folders. `dev-data/` is gitignored and created when missing.

The main window is declared with `"create": false` in `tauri.conf.json` and built in `setup` (`startup/main_window.rs`) so its WebView profile follows the mode.

`npm`/`vite`/`tauri` and `cargo` commands are run from the repository root (`npm run tauri dev`, `cargo test --workspace`).

## Points of attention

- **No automatic offer collection.** Scraping of job boards, filtering, LLM scoring, the scheduler and the Marché tab were removed (they went against several platforms' terms of use); the code is kept at the Git tag `archive/collecte-automatique`. An offer exists only inside its application (`url_offre`, `description_offre`), added by link or pasted text. Don't reintroduce bulk collection from job platforms.
- **No LLM writes Typst during generation.** References are converted once at upload into `template.typ` + `example_data.json` + `reference_texts.json`; generation fills a canonical `data.json` from the profile (facts), LLM-written prose and the reference texts. See `docs/project.md` §7.
- **Templatization has two steps and an imposed model.** Step A fills `example_data` under structured output (keep its schema under the API grammar cap: `tests/integration/llm_schema_size.rs` fails otherwise); step B writes the template as free text. Both always use `TEMPLATIZE_MODEL`, whatever the settings say. The markup is read by Rust before any LLM call — never ask the LLM to extract guidance.
- **The reference is the source of truth for structure and text; the profile for information.** Templatization never removes anything: rendered with its example data, the template gives back the reference's exact words. Every visible text that is neither profile data, a label nor prose is a reference text read from `data.texts.<id>` (translatable), never hard-coded. Information the profile holds replaces the reference's, and nothing ever falls back to the reference, identity included: an empty contact (email, phone, LinkedIn, GitHub, portfolio) or title stays empty and its line disappears, with one French warning per contact shown by the reference's example (`docgen/contact_fields.rs`, deduplicated across the CV and the letter, `GeneratedDocuments.warnings`); an empty first or last name refuses the generation before any LLM call (`AppError::Validation`; the profile cannot be saved with a blank one either). The sender's location (city, postal code) belongs to the session (`sessions.ville` / `sessions.zip_code`, city required to create an application) — lists (skills, languages) never fall back either. The profile has no interests: an « Intérêts » line of a reference is a hard-coded reference text (`data.texts`), translated and cached like any other, and any line that belongs to no profile category is treated the same way. The categories the profile feeds into the CV (`docgen/profile_categories.rs`, the single source) drive the step-A prompt (closed field list) and the generation: a category shown by the reference's example but empty in the profile (experiences, projects, formations, skills, languages, driving licence **not given**) is removed from the CV with a French warning (`GeneratedDocuments.warnings`), never filled from the reference; a driving licence set to « no » is removed without a warning. `[FIXE]` / `[VARIABLE]` / `[ADAPTABLE]` zones live in `reference_texts.json`: a fixed zone is copied word for word, translated faithfully in another language. Generation never modifies a template.
- **The detailed context is never rendered.** `experiences.contexte` / `projets.contexte` (free text, at most 2,000 characters, validated in Rust) is LLM input only: the matching receives the projects' context (never the experiences'), through `llm::profile_prompt`, and `summary_strategy` must not cite a fact found only there; the letter receives every experience's and the selected projects' context in its own block (`llm::detailed_context_block`, absent when nothing is filled in, so the letter prompt is then unchanged). It never enters the CV path (`entry_texts`, `profile_translation`, `CvEntryTexts`, `profile_facts_block`, `cv_writing`, `build_cv_data`), is never translated, and never reaches `candidature_message`. Don't serialize the whole `ProfileDto` into a prompt: go through `profile_prompt_json`. A letter paragraph that repeats 8+ consecutive words of a context is retried (`docgen::verbatim_check`) and, if the copy survives the retries, kept with a warning — never a blocking failure.
- **Documents are written entirely in the offer's language.** The profile's language is declared (`users.langue_profil`); reference texts, labels and profile texts are translated by validated calls and cached per version and language, with the prompt version in the cache (bump `*_PROMPT_VERSION` when a translation prompt changes). Official names (job title, company, degree, school, project title) are never translated. When the languages match, no translation call is made.
- **An old reference version is refused, not migrated.** A version without `reference_texts.json` or with another `format_version` raises `AppError::OutdatedReference` asking to re-import it. The format version is per document type (`models::expected_format_version`: CV = 4, cover letter = 3; the letter's bump forces the re-import of letter references whose recipient block kept a fixed width, the CV's of CV references whose list symbol was hard-coded).
- **The letter quotes the exact job title and splits the location.** `candidatures.poste` is given to the letter writer (`LmWritingInput.job_title`) and the review retries unless the subject contains it (normalized comparison, `docgen/title_match.rs`); the letter is always in the offer's language, so no translation exception. The recipient is built without an LLM (`docgen/recipient.rs::split_location`): `candidatures.localisation` is split on commas, the last part is `recipient.location`, the previous ones `extra_lines` (the stored location is unchanged). By exception to « copy the spacing », the recipient block is as wide as its longest line, flush against the right margin with left-aligned text (`#align(right, box({ set align(left); … }))`, same vertical positions as the `block` it replaces); `documents/recipient_check.rs` checks it on a render with only company and city, only when the reference's block is on the right, tolerated at the last attempt like orphan separators.
- **Descriptions are lists with a dash, the CV symbol belongs to the session.** In experiences, projects and formations, a line starting with `-` is an item (`-` followed by a digit stays text; `•`, `*`, `–`, `—` are text); lines before the first item are the intro, a line without a dash after an item continues it. Two mirror parsers apply the rule, `docgen/description_list.rs` and `src/composables/parseDescriptionList.ts`, tested against the same table `tests/fixtures/description_list_cases.json`: change both and the table together. The app always shows bullets; `sessions.format_liste` (`puce` « • » / `tiret` « - », editable only on an active session) applies only to the generated CV, through `data.list_marker`, which the CV template places in front of every item of every description section (never hard-coded; header separators are not list symbols), checked by a sentinel render (`documents/list_marker_check.rs`). The letter and the prompts receive the flattened text without symbol. Migration `0013` converted the old descriptions (test `db_migration_descriptions_listes.rs`).
- **Statistics are derived, never stored.** No table or migration: `services::session_stats` recomputes everything from `candidatures`, `status_history` and `interviews` at each call. Sent = a `candidature_envoyee` event; a response is the first event after it among `message_recu`, `appel`, `entretien_planifie`, `offre_recue`, `refus_recu` (`timeline::RESPONSE_EVENTS`); the « platform » is `candidatures.source` (stored as a French label, translated by `sourceLabel` on the frontend). The thresholds (`MIN_SENT_FOR_STATS`, `MIN_SENT_PER_ROW`) travel in the DTO — don't hard-code them in Vue, and don't compute a rate or a rule in the frontend beyond rounding.
- **Generation runs in the app process.** `services::generation` publishes progress before `create_candidature` returns, then runs the pipeline as a background task. A failed generation deletes the application and its partial files; a generation cut short by closing the app is cleaned up at the next startup (every `a_postuler` application with no document).
- **A reference version is all or nothing.** Its row is inserted in a write transaction before its files are written; any failure rolls back and removes the folder (`documents/storage.rs::persist_version`).
- **Offer import is user-triggered, one page at a time.** No crawl, no retry loop, no proxy/identity rotation/captcha solving. Only a validated public `http(s)` URL is fetched (`scraping::url_validation`, DNS filtered by `safe_resolver`). Failures are typed (`ScrapingFailure`, serialized as a code in `AppError::Scraping`) and the UI chooses the message — don't hardcode user-facing advice in the backend. Every description, whatever its path, goes through `scraping::finalize::finalize_fields`.
- **The capture window has no IPC access.** The `capture` window is deliberately absent from every capability; its HTML is read from Rust with `eval_with_callback`. Never add it to a capability or give it a `remote` capability, and keep validating the URL before loading it.
- **The API key source follows the run mode.** DEV reads `ANTHROPIC_API_KEY` from the `.env` and never opens the keyring; PROD reads the system keyring (`keyring` crate, `v1` feature only; service = app identifier, account `anthropic_api_key`) and ignores any `.env` key. Code that calls Claude gets its `ClaudeTarget` from `ClaudeTargetSource::resolve`: never read the key elsewhere, never copy it into the environment, never log it, send it to the frontend (only its last 4 characters) or put it in an error. An unavailable keyring is an explicit error, never a fallback to a file. Tests use the in-memory `tests/integration/shared/fake_secret_store.rs`, never the real keyring.
- **Test coverage.** `cargo test` covers HTML/Markdown-to-text conversion, the offer import's pure logic, the documents pipeline's pure logic, the `db` layer on in-memory SQLite and the file-handling services. It also runs service-level tests, at the level the Tauri commands call:
  - offer import by link (real reduced pages served by a local `wiremock` server, typed failures) and by capture (`offer_import.rs`);
  - CV/LM generation with real Typst compilation, empty-prose retries and failure cleanup (`generation.rs`), asymmetric budgets, page warnings and no fallback to the reference (`generation_budgets.rs`), full translation with real marked-up references and caches (`generation_translation.rs`);
  - reference markup (`templatize_markup.rs`), step A/B validation on tagged references (`templatize_validation.rs`) and the two-step templatization loop replaying real failing outputs (`templatize_retries.rs`);
  - the detailed context: never in the CV files, CV prompts or translations, project contexts only in the matching, both in the letter, unchanged letter prompt without context (snapshot), copy retried then warned (`generation_detailed_context.rs`);
  - the size of every structured output schema against the API grammar cap (`llm_schema_size.rs`);
  - fonts: an imported fixture font used and embedded without warning, a missing family failing the compilation (`typst_fonts.rs`), and the font import (subfolders, `.ttc`, WOFF refusal, family names, homonyms) (`font_import.rs`);
  - session statistics: pure computation on a hand-built history and loading from the database (`session_stats.rs`);
  - description lists: the conversion migration on an old-format database (`db_migration_descriptions_listes.rs`), the CV rendered with real Typst in `puce` and `tiret` (`generation_list_format.rs`) and the list-symbol check of the template (`templatize_validation.rs`);
  - profile, session and application-deletion lifecycles;
  - one end-to-end scenario (`end_to_end.rs`).

  The crate has a single integration test binary, `src-tauri/tests/integration/` (`main.rs` declares one module per test file), to keep disk use and link time down. Every new integration test is a module of `tests/integration/`, declared in its `main.rs` — never a new file at the root of `tests/`. All integration tests run in one process: anything shared on disk must be unique per test, not per process.

  Helpers live in `src-tauri/tests/integration/shared/` (the pure-logic tests' shared fixtures in `src-tauri/tests/integration/common/`):
  - `fake_llm.rs` is an `LlmCaller` with canned answers from `tests/fixtures/llm/`, routed by output schema (the template step, without schema, by its system prompt); `Reply::Files` replays raw files, e.g. real failing templates; it records the model of each call;
  - `offer_server.rs` is the local HTTP server;
  - `font_fixtures.rs` copies a fixture font folder (`tests/fixtures/fonts/`) into a test's folder;
  - `temp_dir.rs` gives each test a unique temporary folder;
  - `test_state.rs` builds an `AppState` (in-memory pool, temp data folder, fake keyring, fake LLM), seeds a profile and test references, and waits for a generation.
  - `tagged_references.rs` builds an app with the tagged references templatized, runs a generation and renders its `data.json` with the version's template.

  Fixtures and their sources are listed in `tests/fixtures/README.md`. Loopback fetching exists only in `scraping::fetch::fetch_html_allowing_loopback`, compiled under the crate's `test-support` feature. Only the crate's dev-dependency on itself (`findmyjob = { path = ".", features = ["test-support"] }` in `[dev-dependencies]`) enables that feature, so a plain `cargo test --workspace` has it and no app build does; tests inject it through `services::scraping::scrape_offer_with`.

  No test calls the network, the Claude API or the real keyring, or reads `.env` or `dev-data`. The Tauri commands themselves (thin wrappers) and the capture window are not tested.

## What NOT to do

- Don't reintroduce a local server, an HTTP layer between the frontend and the Rust side, PostgreSQL, or Docker — the server-mode version is archived at the Git tag `archive/agent-postgres`.
- Don't enable another sqlx driver than SQLite, at the workspace or crate level.
- Don't read the clock outside `clock.rs`, and don't build a data-folder path outside `storage_paths.rs`. Don't pick a data, log or WebView folder outside `startup/run_mode.rs` — the rest of the code receives them.
- Don't add database access, Tauri or file-system persistence to the modules `llm`, `scraping`, `text`, `trace_log` or the pure files of `docgen` — they stay pure.
- Don't add business logic to the Vue frontend or to a Tauri command file — it belongs in `db/` or `services/`.
- Don't introduce a configurable AI provider base URL or an OpenAI-compatible abstraction — the provider is deliberately Anthropic-specific today (endpoint, auth headers, and structured-output format are all Claude-specific in `src-tauri/src/llm/client.rs`).

## Key conventions

- **Interface language (i18n).** The frontend is bilingual (French / English), the backend is not:
  - every visible text in `src/` goes through `t()` (`useI18n()` in components, `t` from `@/i18n` elsewhere), `<i18n-t>` when the text contains markup; never a hard-coded string;
  - `src/locales/fr.ts` is the reference; `en.ts` is typed `satisfies typeof fr`, so a missing or extra key fails `vue-tsc`. Every new key gets its French and English text in the same change;
  - values stored in the database stay in French (statuses, sources, language levels…): only their displayed label is translated;
  - the Rust backend is not translated: its messages are French; in English the UI shows a generic translated message followed by the original (`backendErrorMessage`, `backendTextMessage`);
  - the interface language (`ui_lang` setting) is independent of the documents' language: CVs and cover letters always follow the offer's language.
- The app's data (database, documents, fonts) lives in the Tauri app data folder, never inside the project tree.
- The Anthropic API key is read from `ANTHROPIC_API_KEY` in `.env` in DEV mode, and from the system keyring in PROD mode (set from Paramètres). It is never hardcoded, committed or stored in the database. `.env` is gitignored; `.env.example` documents it without a value.
