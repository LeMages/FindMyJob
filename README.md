# FindMyJob

**[English](#english) · [Français](#francais)**

<a id="english"></a>

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

## Usage

The typical path, from first launch to your first tailored application.

### 1. Fill in your basic information

On first launch, a short wizard asks for the interface language (English or French, changeable later in **Settings**), then your identity, contact details, location, a short description and your links (LinkedIn, GitHub, portfolio). Only your first name, last name and email are required; everything can be edited later.

<!-- Screenshot placeholder: onboarding wizard (Identity step). -->

### 2. Add your Anthropic API key

The AI features (offer import, CV and cover letter tailoring, application messages) need an [Anthropic API key](https://console.anthropic.com/). Open **Settings → Artificial intelligence**, paste your key and save it (see [Anthropic API key](#anthropic-api-key)).

<!-- Screenshot placeholder: Settings, Artificial intelligence block. -->

### 3. Complete your profile in detail

Open your profile from the bottom of the sidebar (**View profile**) and fill in your experiences, education, projects, skills and languages, and set the language your profile is written in. The more detail you give, the better the generated documents: the AI only uses what your profile says and never invents facts.

<!-- Screenshot placeholder: profile page in edit mode. -->

### 4. Write your CV and cover letter in Typst with Claude

FindMyJob tailors *your own* documents, so it needs a reference CV and cover letter written in [Typst](https://typst.app/) (`.typ` files). If you don't have them yet, ask [Claude](https://claude.ai/) to convert your current CV and cover letter (PDF or Word) to Typst, then check the result compiles and looks right.

You can add writing instructions as `//` comments in the file (`[CONSIGNES]`, `[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`) to say which parts stay word for word and which are rewritten for each offer. See [`docs/reference-markup.md`](docs/reference-markup.md).

### 5. Create a session and import your documents

A session groups the applications of one search (for example "Internship 2027"). Click **+ New session** in the sidebar and name it. Then open its **Settings** tab and, in the **Documents** card, import your CV and cover letter `.typ` files. Each session has its own reference documents, and they are analyzed once at import.

<!-- Screenshot placeholder: session Settings tab, Documents card with both references imported. -->

### 6. Import the fonts used by your documents

If your CV or cover letter uses fonts that are not built in, import them so the generated PDFs match your originals: download the font (for example from [Google Fonts](https://fonts.google.com/)), unzip it, then select its folder in **Settings → Fonts**. Write in `font:` the exact family name shown there. A document that uses a missing font fails to compile instead of silently falling back.

<!-- Screenshot placeholder: Settings, Fonts block with an imported family. -->

### 7. Add offers and apply

In your session, click **+ New application** and paste the offer's link: the form is filled in automatically for you to review. If the site can't be read, open the page inside FindMyJob, paste its text, or fill the form by hand. On creation, FindMyJob generates a CV and cover letter tailored to the offer, in the offer's language. You then follow the application from the tracking view (status timeline, follow-ups, interviews in the **Planning** tab).

<!-- Screenshot placeholder: New application, offer link step. -->
<!-- Screenshot placeholder: application detail with generated documents. -->

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

- **DEV**: from `ANTHROPIC_API_KEY` in the `.env`. The system keyring is never used; the **Settings** page shows the key's state read-only.
- **PROD**: from the system keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service; entry `com.findmyjob.app` / `anthropic_api_key`). A key in a `.env` is ignored. Open **Settings → Artificial intelligence**, paste the key and click **Save**: the connection test runs right after. The key can be replaced or deleted from the same block.

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

---

<a id="francais"></a>

# FindMyJob (français)

**[English](#english) · [Français](#francais)**

Application de bureau open source pour suivre vos candidatures.

FindMyJob aide les étudiants à organiser leur recherche de stage, d'alternance ou d'emploi, de la première offre repérée jusqu'à la réponse finale. Au-delà du simple suivi, elle utilise l'IA pour supprimer les tâches fastidieuses : remplir les détails d'une offre, rédiger une relance, adapter un CV et une lettre de motivation au poste.

<!-- Screenshot placeholder: capture de la vue de suivi, à ajouter quand elle sera disponible. -->

## Fonctionnalités

- **Suivi des candidatures** — entreprise, poste, type de contrat (stage / alternance / CDI / CDD / intérim / freelance), contacts, notes, historique complet des statuts (candidature, relances, entretiens, offre, issue).
- **CV et lettre de motivation adaptés par l'IA** — générés à partir de votre profil et du texte de l'offre, dans le style de votre propre document, entièrement dans la langue de l'offre (français ou anglais). Les consignes de rédaction sont des commentaires dans votre document de référence, selon un format documenté ([`docs/reference-markup.md`](docs/reference-markup.md)).
- **Message de candidature rédigé par l'IA** — un court email ou message adapté à l'offre au moment de postuler.
- **Import d'offre** — collez le lien d'une offre et le formulaire est prérempli automatiquement (intitulé, entreprise, lieu, type de contrat, rémunération, description complète) pour que vous le vérifiiez. Les données structurées de la page (JSON-LD) sont utilisées quand elles existent, sinon son texte est analysé par l'IA. Si un site refuse la lecture automatique, vous pouvez ouvrir la page dans FindMyJob (accepter les cookies, vous connecter) et la capturer, coller son texte, ou remplir le formulaire à la main. Une candidature existante pour la même offre est signalée.
- **Planning des entretiens** — vues semaine et mois sur toutes vos candidatures.

Chaque offre entre dans l'application par son lien (ou son texte collé) : FindMyJob ne collecte jamais d'offres de lui-même.

## Utilisation

Le parcours type, du premier lancement à votre première candidature adaptée.

### 1. Renseigner vos informations de base

Au premier lancement, un court assistant demande la langue de l'interface (français ou anglais, modifiable ensuite dans les **Paramètres**), puis votre identité, vos coordonnées, votre localisation, une courte description et vos liens (LinkedIn, GitHub, portfolio). Seuls le prénom, le nom et l'email sont obligatoires ; tout reste modifiable par la suite.

<!-- Screenshot placeholder: assistant de première configuration (étape Identité). -->

### 2. Ajouter votre clé API Anthropic

Les fonctions d'IA (import d'offre, adaptation du CV et de la lettre, messages de candidature) nécessitent une [clé API Anthropic](https://console.anthropic.com/). Ouvrez **Paramètres → Intelligence artificielle**, collez votre clé et enregistrez-la (voir [Clé API Anthropic](#clé-api-anthropic)).

<!-- Screenshot placeholder: Paramètres, bloc Intelligence artificielle. -->

### 3. Compléter votre profil en détail

Ouvrez votre profil depuis le bas de la barre latérale (**Voir le profil**) et renseignez vos expériences, formations, projets, compétences et langues, ainsi que la langue dans laquelle votre profil est rédigé. Plus vous donnez de détails, meilleurs sont les documents générés : l'IA n'utilise que ce que dit votre profil et n'invente jamais de faits.

<!-- Screenshot placeholder: page du profil en mode édition. -->

### 4. Rédiger votre CV et votre lettre en Typst avec Claude

FindMyJob adapte *vos propres* documents : il lui faut donc un CV et une lettre de motivation de référence rédigés en [Typst](https://typst.app/) (fichiers `.typ`). Si vous n'en avez pas encore, demandez à [Claude](https://claude.ai/) de convertir votre CV et votre lettre actuels (PDF ou Word) en Typst, puis vérifiez que le résultat compile et s'affiche correctement.

Vous pouvez ajouter des consignes de rédaction en commentaires `//` dans le fichier (`[CONSIGNES]`, `[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`) pour indiquer quelles parties restent mot pour mot et lesquelles sont réécrites pour chaque offre. Voir [`docs/reference-markup.md`](docs/reference-markup.md).

### 5. Créer une session et importer vos documents

Une session regroupe les candidatures d'une même recherche (par exemple « Stage 2027 »). Cliquez sur **+ Nouvelle session** dans la barre latérale et donnez-lui un nom. Ouvrez ensuite son onglet **Paramètres** et, dans la carte **Documents**, importez les fichiers `.typ` de votre CV et de votre lettre. Chaque session a ses propres documents de référence, analysés une seule fois à l'import.

<!-- Screenshot placeholder: onglet Paramètres de la session, carte Documents avec les deux références importées. -->

### 6. Importer les polices utilisées par vos documents

Si votre CV ou votre lettre utilise des polices non intégrées, importez-les pour que les PDF générés correspondent à vos originaux : téléchargez la police (par exemple sur [Google Fonts](https://fonts.google.com/)), décompressez-la, puis sélectionnez son dossier dans **Paramètres → Polices**. Écrivez dans `font:` le nom exact de la famille affiché à cet endroit. Un document qui utilise une police manquante échoue à la compilation au lieu de basculer silencieusement sur une autre.

<!-- Screenshot placeholder: Paramètres, bloc Polices avec une famille importée. -->

### 7. Ajouter des offres et postuler

Dans votre session, cliquez sur **+ Nouvelle candidature** et collez le lien de l'offre : le formulaire est rempli automatiquement pour que vous le vérifiiez. Si le site ne peut pas être lu, ouvrez la page dans FindMyJob, collez son texte, ou remplissez le formulaire à la main. À la création, FindMyJob génère un CV et une lettre adaptés à l'offre, dans la langue de l'offre. Vous suivez ensuite la candidature depuis la vue de suivi (historique des statuts, relances, entretiens dans l'onglet **Planning**).

<!-- Screenshot placeholder: Nouvelle candidature, étape du lien de l'offre. -->
<!-- Screenshot placeholder: détail d'une candidature avec les documents générés. -->

## Architecture technique

FindMyJob est une application de bureau autonome : un seul processus, pas de serveur, pas de Docker. Elle suit l'organisation standard d'un projet Tauri :

```
src/        Frontend Vue 3
src-tauri/  Backend Rust Tauri 2 — gère la base SQLite, les appels au LLM, l'import d'offre, le pipeline des documents et les règles métier
```

Le frontend Vue appelle des commandes Tauri, qui appellent directement la logique métier. Voir [`docs/project.md`](docs/project.md) pour la référence technique complète (modèle de données, pipelines, commandes, conventions).

## Données

Tout est stocké localement, dans le dossier de données : une base SQLite (`findmyjob.db`), vos documents de référence et les PDF générés (`documents/`) et les polices importées (`polices/`). Le schéma est créé et migré automatiquement au démarrage ; voir [`db/schema.dbml`](db/schema.dbml) pour le détail des colonnes.

Le dossier de données dépend du mode d'exécution, défini par `MODE` dans le `.env` :

| Mode | Dossier de données | Logs et profils WebView |
|---|---|---|
| PROD (par défaut) | dossier de données de l'application (sous Windows `%APPDATA%\com.findmyjob.app\`) | dossier local de l'application (sous Windows `%LOCALAPPDATA%\com.findmyjob.app\`, logs dans `logs\findmyjob.log`) |
| DEV | `dev-data/` à côté du `.env` qui déclare `MODE=DEV` (en développement, la racine du dépôt) | même dossier `dev-data/` (`logs/`, `webview/`, `capture_webview/`) |

`MODE` est lu dans le `.env` chargé : pas de `.env`, `MODE` absent ou `MODE=PROD` → PROD ; `MODE=DEV` → DEV ; toute autre valeur est une erreur au démarrage (la casse est ignorée). En mode DEV, rien n'est écrit dans les dossiers système de l'application. `dev-data/` est ignoré par Git et créé s'il manque.

## Prérequis

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- Dépendances système de Tauri : [guide officiel](https://tauri.app/start/prerequisites/)
- Une [clé API Anthropic](https://console.anthropic.com/) pour les fonctions d'IA
- Sous Linux, un Secret Service actif (GNOME Keyring, KWallet) pour stocker la clé en mode PROD

## Démarrage

```bash
git clone https://github.com/KanASon/FindMyJob.git
cd FindMyJob
cp .env.example .env   # MODE=DEV pour un dossier dev-data/ isolé, et renseigner ANTHROPIC_API_KEY
npm install
npm run tauri dev
```

En développement, le `.env` à la racine du dépôt est trouvé automatiquement. Une application installée le lit dans son dossier de configuration (sous Windows `%APPDATA%\com.findmyjob.app\.env`). Les logs sont écrits dans `findmyjob.log`, dans le dossier de logs du mode d'exécution (voir [Données](#données)) ; la première ligne au démarrage indique le mode et les dossiers utilisés.

### Clé API Anthropic

L'endroit où la clé est lue dépend du mode d'exécution :

- **DEV** : depuis `ANTHROPIC_API_KEY` dans le `.env`. Le trousseau système n'est jamais utilisé ; la page **Paramètres** affiche l'état de la clé en lecture seule.
- **PROD** : depuis le trousseau système (Gestionnaire d'identification Windows, Trousseau macOS, Secret Service Linux ; entrée `com.findmyjob.app` / `anthropic_api_key`). Une clé présente dans un `.env` est ignorée. Ouvrez **Paramètres → Intelligence artificielle**, collez la clé et cliquez sur **Enregistrer** : le test de connexion se lance juste après. La clé peut être remplacée ou supprimée depuis le même bloc.

La clé complète n'atteint jamais l'interface (seulement ses 4 derniers caractères) et n'apparaît jamais dans les logs ni dans les messages d'erreur. Si le trousseau est indisponible (par exemple Linux sans Secret Service), l'application le signale explicitement ; il n'y a pas de repli sur un fichier.

### Build de production

```bash
npm run tauri build
```

L'installateur est généré dans `target/release/bundle/` à la racine du dépôt.

### Tests

```bash
cargo test --workspace
```

À lancer depuis la racine du dépôt.

## Stack technique

| Couche | Technologie |
|---|---|
| Application de bureau | Tauri 2 (Windows / macOS / Linux) |
| Frontend | Vue 3 + TypeScript + Vite + Tailwind CSS |
| Backend | Rust, dans l'application Tauri |
| Base de données | SQLite, via sqlx |
| Génération des documents | Typst (`typst-as-lib`) |
| IA | API Messages d'Anthropic (Claude) |

## Licence

[MIT](LICENSE)

## État du projet

Développement actif, à un stade précoce. Le suivi des candidatures (avec un carnet de contacts recruteurs partagé entre les candidatures), l'adaptation des documents par l'IA et l'import d'offre par lien fonctionnent de bout en bout — voir [`docs/project.md`](docs/project.md#11-features--implemented-partial-and-planned) pour la vue d'ensemble.

## Historique

Une première version collectait les offres automatiquement (Welcome to the Jungle, filtrage, notation par LLM). Je l'ai retirée après avoir réalisé que cette approche allait à l'encontre des conditions d'utilisation de plusieurs plateformes d'emploi. L'application repose désormais sur l'ajout des offres par leur lien. Le pipeline d'origine reste disponible au tag `archive/collecte-automatique`.

Jusqu'en octobre 2026, l'application fonctionnait en mode serveur : un agent Axum local avec PostgreSQL dans Docker, l'application de bureau n'étant qu'un client HTTP léger. C'est désormais une application de bureau autonome avec une base SQLite intégrée ; la version serveur est conservée au tag `archive/agent-postgres`.

## À propos

Je suis étudiant en école d'ingénieur informatique et, comme beaucoup, j'ai du mal à trouver un stage. Entre des dizaines de candidatures à suivre, des relances à ne pas oublier et des formulaires à remplir à la main, j'ai fini par construire mon propre outil pour garder le cap. J'ai décidé de le rendre open source pour aider les personnes dans la même situation.
