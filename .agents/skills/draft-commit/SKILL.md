---
name: draft-commit
description: Analyze staged git changes and draft a commit with a structured message following project conventions
disable-model-invocation: true
allowed-tools: Bash(git *)
---

## Context

Analyze the staged git changes, or those of a commit if the user
names one, and draft a commit message for the user to commit.

## Step 1: Gather context

Run these commands in parallel:

- `git status` to see staged and unstaged files (never use `-uall`)
- `git diff --cached` to see the staged diff
- `git diff --cached --stat` for a file summary
- `git log --oneline -10` for recent commit style reference
- `cat .gitmessage` for the message format

If nothing is staged, tell the user and stop. Ask the user which files
to stage if there are unstaged changes.

## Step 2: Draft the message

Write the message by `.gitmessage`, the single source of truth for
the tag, summary, body and sign-off. Pick the tag from the changed
paths by its tag mapping. Name the change with a precise verb (Add,
Fix, Remove, Extract, Compute, Wire) over a vague one such as
"update" or "change".

The draft is done when the summary, tag included, fits in 50
characters and every body line wraps at 72.

## Step 3: Hand over

Print the message and let the user commit it with `git commit -s`:

```
tag: Summary here

Body here.
```
