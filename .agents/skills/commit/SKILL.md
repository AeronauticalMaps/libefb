---
name: commit
description: Analyze staged git changes and create a commit with a structured message following project conventions
disable-model-invocation: true
allowed-tools: Bash(git *)
---

## Context

Analyze the staged git changes or those of a commit if specified by
the use. Draft a commit message following the project's conventions,
and draft the commit message.

## Step 1: Gather context

Run these commands in parallel:

- `git status` to see staged and unstaged files (never use `-uall`)
- `git diff --cached` to see the staged diff
- `git diff --cached --stat` for a file summary
- `git log --oneline -10` for recent commit style reference

If nothing is staged, tell the user and stop. Do not stage files
automatically — ask the user which files to stage if there are unstaged
changes.

## Step 2: Determine the module tag

Infer the tag from the changed file paths using this mapping:

| Path prefix             | Tag            |
|-------------------------|----------------|
| `efb/src/route/`        | `route`        |
| `efb/src/fp/`           | `fp`           |
| `efb/src/measurements/` | `measurements` |
| `efb/src/nd/`           | `nd`           |
| `efb/src/fms/`          | `fms`          |
| `efb/src/core/`         | `core`         |
| `efb/src/aircraft/`     | `aircraft`     |
| `efb/src/geom/`         | `geom`         |
| `efb/src/fc/`           | `fc`           |
| `arinc424/`             | `arinc424`     |
| `aixm/`                 | `aixm`         |
| `handbook/`             | `handbook`     |
| `bindings/c/`           | `c`            |
| `bindings/wasm/`        | `wasm`         |
| `bindings/python/`      | `python`       |
| `bindings/swift/`       | `swift`        |

Rules:

- If all meaningful changes fall within **one module**, use that tag.
- If binding changes merely mirror a core module change (e.g. updating
  a C binding wrapper because the Rust API changed), use the **core
  module's** tag.
- If changes span **multiple unrelated modules**, omit the tag entirely.
- Files like `CLAUDE.md`, `CONTRIBUTING.md`, `.github/`, `Cargo.toml`
  are not module-specific — ignore them when determining the tag unless
  they are the only changes (then omit the tag).

## Step 3: Draft the commit message

Follow these formatting rules:

### Title line

- Format: `tag: Imperative summary` (or just `Imperative summary` if
  no tag)
- **Maximum 50 characters** including the tag and colon
- Use **imperative mood**: Add, Fix, Remove, Refactor, Implement,
  Wire, Compute, Extract
- **Never** use vague words: "update", "change", "stuff", "misc",
  "various"
- Capitalize the first word after the tag

### Body (optional, use when the diff is non-trivial)

- Separate from the title with **one blank line**
- Wrap lines at **72 characters**
- Explain the **why**, not the what — the diff shows the what
- Use bullet points for multiple distinct changes
- If there is a breaking change, add `BREAKING CHANGE:` followed by a
  description

## Step 4: Create the commit message

- Simply print the commit message but let the user do the commit:

```
tag: Title here

Body here.
```
