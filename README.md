# FindMyJob

Open-source desktop app to track your job applications.

FindMyJob helps students organize their search for an internship, apprenticeship, or full-time job, from the first offer they spot to the final answer. Beyond plain tracking, it uses AI to remove the tedious parts: filling in an offer's details, drafting a follow-up, tailoring a CV and cover letter to the job.

<!-- Screenshot placeholder: add a screenshot of the tracking view here once available. -->

## Features

- **Application tracking** — company, position, contract type (internship / apprenticeship / permanent / fixed-term / temp / freelance), contacts, notes, full status timeline (applied, follow-ups, interviews, offer, outcome).
- **AI-tailored CV and cover letter** — generated from your profile and the offer's text, in your own document's style, entirely in the offer's language (French or English). Writing instructions are comments in your reference document, in a documented format ([`docs/reference-markup.md`](docs/reference-markup.md)).
- **AI-drafted application message** — a short email or message adapted to the offer when you apply.
- **Offer import** — paste a job listing's URL and the form is prefilled automatically (title, company, location, contract type, pay, full description) for you to review. The page's structured data (JSON-LD) is used when available, otherwise its text is analyzed by the AI. If a site refuses automatic reading, you can open the page inside FindMyJob (accept cookies, log in), capture it, paste its text, or fill the form by hand. An existing application for the same offer is flagged.
- **Interview planning** — week/month views across all your applications.

Every offer enters the app through its link (or its pasted text): FindMyJob never collects offers on its own.

## Architecture

FindMyJob is a standalone desktop app: one process, no server, no Docker. It follows the standard Tauri layout:

```
src/        Vue 3 frontend
src-tauri/  Tauri 2 Rust backend — owns the SQLite database, the LLM calls, the offer import, the documents pipeline and the business rules
```

The Vue frontend calls Tauri commands, which call the business logic directly. See [`docs/project.md`](docs/project.md) for the full technical reference (data model, pipelines, commands, conventions).

## Data

Everything is stored locally, in the data folder: a SQLite database (`findmyjob.db`), your reference documents and generated PDFs (`documents/`) and imported fonts (`polices/`). The schema is created and migrated automatically at startup; see [`db/schema.dbml`](db/schema.dbml) for the column-level reference.

The data folder depends on the run mode, set by `MODE` in `.env`:

| Mode | Data folder | Logs and WebView profiles |
|---|---|---|
| PROD (default) | app data folder (on Windows `%APPDATA%\com.findmyjob.app\`) | app local folder (on Windows `%LOCALAPPDATA%\com.findmyjob.app\`, logs in `logs\findmyjob.log`) |
| DEV | `dev-data/` next to the `.env` that declared `MODE=DEV` (in development, the repository root) | same `dev-data/` folder (`logs/`, `webview/`, `capture_webview/`) |

`MODE` is read from the `.env` that was loaded: no `.env`, `MODE` absent or `MODE=PROD` → PROD; `MODE=DEV` → DEV; any other value is a startup error (case is ignored). In DEV mode nothing is written to the system app folders. `dev-data/` is gitignored and created when missing.

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- Tauri system dependencies: [official guide](https://tauri.app/start/prerequisites/)
- An [Anthropic API key](https://console.anthropic.com/) for the AI features
- On Linux, a running Secret Service (GNOME Keyring, KWallet) to store the key in PROD mode

## Getting started

```bash
git clone https://github.com/KanASon/FindMyJob.git
cd FindMyJob
cp .env.example .env   # set MODE=DEV for an isolated dev-data/ folder, and fill in ANTHROPIC_API_KEY
npm install
npm run tauri dev
```

In development, the `.env` at the repository root is found automatically. An installed app reads it from its config folder (on Windows `%APPDATA%\com.findmyjob.app\.env`). Logs are written to `findmyjob.log` in the log folder of the run mode (see [Data](#data)); the first startup line gives the mode and the folders in use.

### Anthropic API key

Where the key is read depends on the run mode:

- **DEV**: from `ANTHROPIC_API_KEY` in the `.env`. The system keyring is never used; the Paramètres page shows the key's state read-only.
- **PROD**: from the system keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service; entry `com.findmyjob.app` / `anthropic_api_key`). A key in a `.env` is ignored. Open **Paramètres → Intelligence artificielle**, paste the key and click **Enregistrer**: the connection test runs right after. The key can be replaced or deleted from the same block.

The full key never reaches the interface (only its last 4 characters) and never appears in logs or error messages. If the keyring is unavailable (for example Linux without Secret Service), the app reports it explicitly; there is no fallback to a file.

### Production build

```bash
npm run tauri build
```

The installer is generated under `target/release/bundle/` at the repo root.

### Tests

```bash
cargo test --workspace
```

Run from the repository root.

## Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3 + TypeScript + Vite + Tailwind CSS |
| Backend | Rust, inside the Tauri app |
| Database | SQLite, via sqlx |
| Document generation | Typst (`typst-as-lib`) |
| AI | Anthropic Claude Messages API |

## License

[MIT](LICENSE)

## Project status

Active early-stage development. Application tracking (including a global recruiter-contact book shared across applications), AI document tailoring, and offer import by link are implemented end to end — see [`docs/project.md`](docs/project.md#11-features--implemented-partial-and-planned) for the full picture.

## History

An early version collected offers automatically (Welcome to the Jungle, filtering, LLM scoring). I removed it after realizing this approach went against the terms of use of several job platforms. The app now relies on adding offers by their link. The original pipeline is still available at the `archive/collecte-automatique` tag.

Until October 2026, the app ran in server mode: a local Axum agent with PostgreSQL in Docker, the desktop being a thin HTTP client. It is now a standalone desktop application with an embedded SQLite database; the server-mode version is kept at the `archive/agent-postgres` tag.

## About

I'm a computer engineering student and, like many, struggling to find an internship. Between dozens of applications to track, follow-ups to remember, and forms to fill by hand, I ended up building my own tool to stay on top of it. I decided to open-source it to help people in the same situation.
