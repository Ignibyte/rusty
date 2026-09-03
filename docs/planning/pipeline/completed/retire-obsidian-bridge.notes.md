---
title: Retire the Obsidian bridge: notes
pipeline_id: 6bba8f0f-df4a-4d8c-ad83-22c33f553f1e
---

# Retire the Obsidian bridge: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-workspace-is-obsidian-001` (the workspace replaces the
  bridge), `AD-rusty-lenient-pages-001` (the vault format stays Obsidian's); completed
  notes of TICKET-002 (`brain_rename`, `brain_unresolved`, `brain_get_links` cover the
  bridge's reads and its rename); wiki `mcp-back-end.md` (the bridge as a tool family
  and a failure mode), `vault-and-brain.md`, `development-and-validation.md`; the
  brain's `projects/rusty-v3` timeline. A `grep -ri obsidian` over the crates, the
  scripts and the docs listed every touchpoint.
- Decisions: the three in the spec.
- Seal: Chad's goal of 2026-09-02.

## Phase 2: Design

- Architecture and data flow: nothing new; the server loses a handle and six tools,
  the CLI a command, the installer a step, the app a shell call.
- File manifest:

| File | Purpose |
|---|---|
| `crates/rusty-core/src/obsidian.rs`, `src/lib.rs` | the module goes |
| `crates/rusty-mcp/src/main.rs` | the tools, their params, the handle, `EXPECTED` |
| `crates/rusty-mcp/tests/smoke.rs` | the status call goes |
| `crates/rusty-cli/src/main.rs` | the `obsidian` command and its help |
| `crates/rusty-app/src/theme.rs` | the `obsidian configure` call goes |
| `omarchy/install.sh` | the registration and the CLI check go |
| `scripts/screenshot.sh` | `RUSTY_OBSIDIAN_CLI` goes |
| `README.md`, `docs/architecture.md`, `ROADMAP.md`, `openwiki/` | the bridge is gone; the vault is an Obsidian vault by format |

- Store consequences: none.
- Tool contract: `obsidian_status`, `obsidian_open`, `obsidian_backlinks`,
  `obsidian_links`, `obsidian_unresolved` and `obsidian_rename_page` are removed (a
  versioned break); `brain_get_links`, `brain_unresolved` and `brain_rename` are the
  replacements, and the app opens pages.
- Regression plan: REQ-001 the router test and the smoke test (both name every tool);
  REQ-002 and REQ-003 the build; REQ-004 by reading; REQ-005 `grep -ri obsidian` over
  the docs, every hit read.
- Risks: a dependency that only the module used stays in `Cargo.toml` (check
  `cargo build` warnings and the manifest); a document that still promises the bridge.
- CodeGraph evidence: `Obsidian::new` (the server, the CLI), `configure_vault` (the CLI
  only; the app reaches it through the CLI).

## Phase 3: Implement

- Built: the manifest, as removals. `crates/rusty-core/src/obsidian.rs` and its `pub
  mod`; in the server the import, the three parameter types, the `Obsidian` handle and
  its construction, the six tools and their six names in `EXPECTED`; in the smoke test
  the `RUSTY_OBSIDIAN_CLI` line, the name in the tool list and the status call; in the
  CLI the import, four help lines, the dispatch arm and `run_obsidian`; in the app the
  `rusty-cli obsidian configure` call in `Theme::reload`; in the installer the header,
  the CLI check and the registration step; in the screenshot script the environment
  variable. README (status, installer, tool count, the Obsidian section), ROADMAP
  (TICKET-006 with the reason), `docs/architecture.md` (a dated retirement note, the
  as-built lines), the wiki (`mcp-back-end`, `development-and-validation`,
  `quickstart`, the brief).
- Deviations: none. `sha2` stays in `Cargo.toml`: the brain's content hash uses it.
- Fast gate: the full gate below; `grep -ri obsidian_` over the crates, the scripts and
  the docs finds only the retirement notes.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | nothing else in the crates reached the module: the two `use` lines and the app's shell call were the only callers | ok | by `grep` and the build |
| 2 | contract | six tools gone is a versioned break; the README, the wiki and the roadmap name the replacements | ok | documented |
| 3 | data safety | the vault's `.obsidian/` folder and Obsidian's own config are untouched; nothing unregisters the vault | ok | by reading |
| 4 | dependencies | `sha2` and `tokio::process` have other users; no dependency became dead | ok | `cargo build` clean |
| 5 | docs | the architecture document's plan sections keep the 2026-09-02 wording as a record; the dated bullet and the as-built lines say what is true now | ok | by reading |
| 6 | prose | the new passages against `no-ai-slop` | ok | clean |

## Phase 4: Validate

- Tests run (commands and output): the gate ran `cargo test --workspace`: the router
  test (every tool once, every tool described) and the smoke test (list tools, a task
  group, resources, the workspace walk) pass without the bridge; core and app tests
  unchanged.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff] (receipt 2026-09-03T05:53:12Z).
- Smoke evidence: `rusty-cli --help` names no `obsidian` command; the screenshot script
  runs without the variable (the scenes of TICKET-005 came from the same script).
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (router test, smoke, `grep`); REQ-002 PASS (build,
  `grep`); REQ-003 PASS (build, help text); REQ-004 PASS (installer and `Theme::reload`
  by reading, build); REQ-005 PASS (`grep -ri obsidian` over the docs, every hit read).
- Wiki: `update` run through the lifecycle, openwiki_finish returned status complete (receipt 2026-09-03T05:53:42Z); `mcp-back-end` and
  `development-and-validation` reconciled.
- Docs: README, ROADMAP, `docs/architecture.md`, this pair.
- AAR: `docs/planning/knowledge/aar/AAR-006-retire-obsidian-bridge.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | a bridge retired whole, with its config writers | AD-rusty-bridge-retired-whole-001 |
