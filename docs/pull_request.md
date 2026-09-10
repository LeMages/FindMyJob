# Pull Request Preparation Guide

## Purpose

This document defines how to analyze a branch's changes and generate a ready-to-use PR content file. **You must NEVER create the PR yourself — only produce a `PR.md` file at the project root with the title and description for me to copy-paste.**

## Workflow

When asked to prepare a PR, follow these steps in order:

### 1. Identify the branch context

```bash
git branch --show-current
git log --oneline main..HEAD
```

Determine:
- Current branch name
- Target branch (default: `dev`, ask me if unclear)
- Number of commits since divergence

### 2. Analyze the full diff against target

```bash
git diff main..HEAD --stat
git diff main..HEAD
```

If the diff is large, break it down:

```bash
git diff main..HEAD -- <file>
```

### 3. Read the commit history

```bash
git log main..HEAD --pretty=format:"%h %s" --reverse
```

Use the commit messages to understand the intent and progression of changes.

### 4. Generate the PR.md file

Create a `PR.md` file at the project root with the following structure:

```markdown
# <type>(<scope>): <concise title>

## Summary

<!-- 2-3 sentences explaining WHAT this PR does and WHY -->

## Changes

<!-- Grouped list of changes by area, each item explains what was done -->

### <Area 1>
- Description of change
- Description of change

### <Area 2>
- Description of change

## Technical Notes

<!-- Optional: implementation details, trade-offs, things to watch out for -->

## Screenshots

<!-- Optional: placeholder if UI changes are involved -->
<!-- Add: "⚠️ UI changes — add screenshots before merging" -->
```

## PR Title Convention

The PR title follows the same Conventional Commits format as commit messages:

| Type       | Usage                                                  |
|------------|--------------------------------------------------------|
| `feat`     | New feature visible to the user                        |
| `fix`      | Bug fix                                                |
| `refactor` | Code change that neither fixes a bug nor adds a feature|
| `style`    | Formatting, whitespace (no logic change)               |
| `docs`     | Documentation only                                     |
| `test`     | Adding or updating tests                               |
| `chore`    | Build, CI, tooling, dependencies                       |
| `perf`     | Performance improvement                                |
| `ci`       | CI/CD pipeline changes                                 |
| `build`    | Build system or external dependencies                  |

### Title rules

- Imperative mood, lowercase, no period, max 72 characters
- Scope reflects the main area affected: `kanban`, `applications`, `cv`, `ui`, `db`, `api`, `config`, `deps`
- If the PR covers multiple areas, use the most dominant one or omit the scope
- Breaking changes: add `!` → `feat(api)!: redesign application schema`

### Examples

```
feat(kanban): add drag-and-drop between columns
fix(db): handle null status on application insert
refactor(api): extract validation logic into dedicated module
chore(deps): update tauri to v2.2
```

## Description Guidelines

- **Summary**: explain the *what* and the *why*, not the *how*
- **Changes**: group by area of the codebase (frontend, backend, database, config…), not by file
- **Technical Notes**: only if there are non-obvious decisions, trade-offs, or migration steps
- **Screenshots placeholder**: add the reminder if any Vue component or UI layout was touched
- Write in English
- Be specific — no vague statements like "improved the code" or "various fixes"

## Reminders

- ❌ **NEVER create the PR on GitHub or run `gh pr create`**
- ❌ **NEVER commit or push anything**
- ✅ Only produce the `PR.md` file at the project root
- ✅ If the branch has a mix of unrelated changes, flag it and ask me if I want to split
- ✅ If the intent behind certain commits is unclear, **ask me before writing the description**