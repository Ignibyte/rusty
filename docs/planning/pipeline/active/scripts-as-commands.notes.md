---
title: Scripts as commands: notes
pipeline_id: 03f21d1e-614d-4710-9298-72b4dd6f8851
---

# Scripts as commands: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: the draft came from the rustal session on 2026-09-03 with Chad's words of 14:40;
  its facts checked: `usb-reset.sh` sits beside `dev-box-usb` in the store (commit
  28ec440), is linked as `~/.local/bin/usb-reset`, and the ops handbook records it
  (omarchy-ops 5efdc3a). Register: `AD-rusty-mcp-only-back-end-001` (the store is reached
  through the managers and the server); `AD-rusty-agents-are-terminals-001` (a Run action
  that opens a terminal tab fits; an in-process runner would not). Nearest notes:
  TICKET-005 (a tool with a store seam) and TICKET-009 (`rusty-session` stays an installed
  file, not a store script). The skills manager (`crates/rusty-core/src/skills/mod.rs`)
  already has the shapes this needs: a git-backed store, a scan, pending and active states,
  approve and reject; `rusty-cli skills` has the subcommand shape. The app binary today
  takes no subcommands (`crates/rusty-app/src/main.rs`).
- Decisions: two locked in the spec; four open for Chad's seal. No code before the seal.

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
