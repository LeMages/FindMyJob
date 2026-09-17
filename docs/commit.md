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

### 3. Prepare a single commit

All staged changes go into **one single commit**. Do not split into multiple commits — group everything into one logical unit.

### 4. Provide the commit command

Provide the full ready-to-execute command (staging + commit) and **copy it to the clipboard**:

```bash
git add <files> && git commit -m "<type>(<scope>): <message>"
```

Add a `-m "<body>"` only if the subject line isn't self-explanatory.

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
- ❌ **NEVER add co-authorship mentions** (`Co-authored-by`, `Co-written-by`, etc.) in commit messages or trailers — the commit is authored solely by the user
- ✅ Always provide **one single commit** for all staged changes
- ✅ Provide the full command (git add + git commit) and **copy it to the clipboard**
- ✅ If something is unclear about the intent of a change, **ask me before proposing a commit message**