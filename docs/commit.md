# Commit Preparation Guide

## Purpose

This document defines how to analyze staged/unstaged changes and prepare commit messages. **You must NEVER run `git commit` or `git push` — only provide the commit text for me to execute manually.**

## Workflow

When asked to prepare commits, follow these steps in order:

### 1. Analyze the current state

```bash
git status
git diff --stat
git diff --cached --stat
```

### 2. Inspect the actual changes

```bash
git diff
git diff --cached
```

If the diff is large, break it down file by file:

```bash
git diff -- <file>
```

### 3. Identify logical units

Group related changes into **separate commits**. One commit = one logical change. Never bundle unrelated changes together.

Examples of logical units:
- A new feature (component + its styles + its types)
- A bug fix (the fix + its related adjustments)
- A refactor of a single module
- A dependency update

### 4. Provide the commit content

For each commit, give me:

```
Type: <conventional commit type>
Scope: <scope>
Message: <full commit message>
Body (if needed): <description>
```

And the ready-to-paste command:

```bash
git commit -m "<type>(<scope>): <message>" -m "<body>"
```

If files need to be staged selectively, tell me which files to stage for each commit:

```bash
git add <file1> <file2>
```

## Conventional Commits Convention

### Types

| Type       | Usage                                                  |
|------------|--------------------------------------------------------|
| `feat`     | New feature visible to the user                        |
| `fix`      | Bug fix                                                |
| `refactor` | Code change that neither fixes a bug nor adds a feature|
| `style`    | Formatting, semicolons, whitespace (no logic change)   |
| `docs`     | Documentation only                                     |
| `test`     | Adding or updating tests                               |
| `chore`    | Build, CI, tooling, dependencies                       |
| `perf`     | Performance improvement                                |
| `ci`       | CI/CD pipeline changes                                 |
| `build`    | Build system or external dependencies                  |

### Scope

The scope should reflect the area of the codebase affected. Examples for this project:
- `kanban`, `applications`, `cv`, `ui`, `db`, `api`, `config`, `deps`

### Rules

- **Language**: commit messages in English
- **Subject line**: imperative mood, lowercase, no period at the end, max 72 characters
- **Body**: explain *what* and *why*, not *how* — only if the subject line isn't self-explanatory
- **Breaking changes**: add `!` after the scope → `feat(api)!: redesign application schema`

### Examples

```
feat(kanban): add drag-and-drop between columns
fix(db): handle null status on application insert
refactor(api): extract validation logic into dedicated module
chore(deps): update tauri to v2.2
docs(readme): add development setup instructions
style(ui): fix inconsistent spacing in sidebar
```

## Reminders

- ❌ **NEVER run `git commit` or `git push`**
- ✅ Only provide the text and commands for me to copy-paste
- ✅ If changes should be split into multiple commits, specify the staging order
- ✅ If something is unclear about the intent of a change, **ask me before proposing a commit message**