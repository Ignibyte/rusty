---
title: Secrets behind a PIN: notes
pipeline_id: 3b0bcb0e-430a-4733-ac06-1f7ff3b35104
---

# Secrets behind a PIN: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices). Register: `AD-rusty-mcp-only-back-end-001` (the app
  reaches the store only through the back end), §10 (nothing personal ships; nothing
  leaves the machine). Code read: `SecretsManager` (`list`, `get`, `set`, `delete`; the
  file written with mode 0600), the three `secret_*` tools, `SecretsPage.qml` (names,
  set, delete; "a value is typed once and never shown again"); the app never touches the
  file today; `sha2` is a dependency of `rusty-core`, `argon2` is not.
- Decisions: two locked in the spec; three open for the seal, with recommendations.
  No code before the seal.

## Phase 2: Design

- Architecture and data flow:
- File manifest:
- Store consequences:
- Tool contract:
- Regression plan:
- Risks:
- CodeGraph evidence:

## Phase 3: Implement

- Built:
- Deviations:
- Fast gate:

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|

- Post-implementation CodeGraph:

## Phase 4: Validate

- Tests run (commands and output):
- Gate run:
- Smoke evidence:
- Skips or pre-existing failures:

## Phase 5: Complete

- Requirement audit:
- Docs:
- AAR:
- Brain capture:
- Archive:

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
