---
title: Notes into the vault: notes
pipeline_id: 01151524-3b0d-40a5-abd2-8f53a3bff0e9
---

# Notes into the vault: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices). Register: `AD-rusty-files-are-the-truth-001`,
  `AD-rusty-lenient-pages-001` (a file without frontmatter is a page typed by its top
  folder or `note`), `AD-rusty-mcp-only-back-end-001`,
  `PR-rusty-probes-use-throwaway-rows-001` (probes never touch real data). Code read:
  `NotesManager` (`with_root`, `list_tree`, `read_note`, `save_note`, `create_note`,
  `delete_note` soft-deletes into `.deleted/`, `rename_note`, `resolve_safe_path`; eleven
  tests on a temporary directory); `core.rs` builds the manager from the `notes_path`
  setting with `~/.rusty/notes` as the default; six notes tools in `rusty-mcp`; the CLI
  has no `notes` subcommand yet; the `/note` skill in the store reads the setting from
  SQLite and falls back to `~/.rusty/notes`. On the box the folder holds five notes and a
  `.deleted/` pair; the vault has no `notes/` folder yet.
- Decisions: the three locked decisions in the spec. The adoption is never run on Chad's
  notes by this pipeline.

## Phase 2: Design

- Architecture and data flow: the notes manager keeps its shape; only its default root
  moves. `Core::init` resolves the vault path first and defaults `notes_path` to
  `<vault>/notes`; an explicit setting still wins. The vault already types a page under
  `notes/` as `note` (`type_for_folder("notes")`), so the index, search, links, graph and
  embeddings cover the folder with no brain change. The watcher skips the separate notes
  watch when the notes root lies inside the vault, so a change fires once. The adoption
  is `notes::adopt(from, into, dry_run)` in `rusty-core`: it walks the old folder, refuses
  when any destination exists (moving nothing), renames each file into place (copy and
  remove across devices), deletes nothing, skips a README of its own from an earlier run,
  and writes that README naming the new place. `rusty-cli notes adopt [--dry-run]` calls
  it from the legacy root (the `notes_path` setting when set, else `~/.rusty/notes`) into
  `<vault>/notes`, then sets `notes_path` to the new folder so every consumer follows;
  `rusty-cli notes path` prints the resolved folder.
- File manifest:
  - `crates/rusty-core/src/notes/mod.rs`: `AdoptReport`, `adopt`, `ADOPT_README`, four tests.
  - `crates/rusty-core/src/core.rs`: the default notes root inside the vault.
  - `crates/rusty-core/src/lib.rs`: the watcher skips a notes root inside the vault.
  - `crates/rusty-cli/src/main.rs`: `notes adopt [--dry-run]`, `notes path`, the usage.
  - `crates/rusty-app/qml/SettingsPage.qml`: the `notes_path` entry's fallback text.
  - `omarchy/install.sh`: the one-shot line.
  - The store's `/note` skill (outside this repo, on the box): the fallback path.
  - Phase 5: `README.md`, `ROADMAP.md`, `docs/architecture.md`, the wiki page
    `mcp-back-end.md`.
- Store consequences: the notes folder's default location changes; no schema change; the
  vault gains a `notes/` folder (created by the manager at start). An existing
  `notes_path` setting keeps its meaning. Chad's real notes move only when he runs the
  one-shot.
- Tool contract: none. The six notes tools keep their names and parameters; their paths
  are relative to the root, which moves.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | `adopt_moves_files_and_folders_and_leaves_a_readme`, `adopt_refuses_a_clash_and_moves_nothing`, `adopt_dry_run_moves_nothing` on temporary folders |
  | REQ-002 | `core.rs` by reading (the default is `<vault>/notes`); the scratch scene shows the folder |
  | REQ-003 | `notes_written_into_the_vault_index_as_note_pages`: a note saved through `NotesManager` rooted at `<vault>/notes`, then `sync_all`, reads back as a page of type `note` |
  | REQ-004 | `adopt_moves_files_and_folders_and_leaves_a_readme` (the README, the second run, nothing removed) |
  | REQ-005 | the `reading` scene against the scratch vault (a `notes` folder in the tree, no special casing) |
  | REQ-006 | `README.md` and the installer's printed line |
- Risks: data safety, the whole ticket. The move refuses on any clash, moves nothing on
  refusal, deletes nothing, and never runs by itself; the tests use temporary folders;
  this pipeline never runs it on the box's notes. A user with an explicit `notes_path`
  keeps it until they adopt. Concurrency: the adoption runs in the CLI while the service
  may watch the folders; the watcher only emits `DataChanged`.
- CodeGraph evidence: `NotesManager` is built in `core.rs` alone and read by the six
  tools in `rusty-mcp`; `start_data_watcher` has one caller (`rusty-mcp` main);
  `type_for_folder` and `type_for_slug` in `vault.rs` already map `notes` to `note` (their
  tests at lines 598 and 599).

## Phase 3: Implement

- Built: `notes::adopt` with `AdoptReport`, `ADOPT_README`, `collect_files` and
  `is_adopt_readme` plus four tests; the default notes root inside the vault in
  `core.rs`; the watcher skipping a notes root inside the vault in `lib.rs`; `rusty-cli
  notes path` and `notes adopt [--dry-run]` with `configured_notes_path` and
  `legacy_notes_path`, the setting pointed at the new folder after a real move and the
  GUI refreshed; the Settings entry's text; the installer's line; the store's `/note`
  skill fallback (commit 21928f3 in the store); the index test
  `notes_written_into_the_vault_index_as_note_pages`.
- Deviations: the adoption points `notes_path` at the new folder rather than clearing the
  setting (the settings manager has no delete), which reads the same and survives a
  future default change. Empty folders under the old root stay, since the ticket says
  nothing is deleted.
- Fast gate: `cargo test -p rusty-core notes::` (15 passed, the four adoption tests
  included); `cargo test -p rusty-core notes_written_into` (1 passed); `cargo build
  --workspace` clean; `bin/gate.sh --fast` on 2026-09-03: `GATE GREEN [fast]`.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | data safety | a second `adopt` run would carry the README it wrote into the vault | medium | fixed while writing: a root `README.md` starting "Notes moved to" is skipped; covered by the first test's second run |
| 2 | data safety | a clash on any file must move nothing | high | by design: every destination is checked before the first rename; covered by `adopt_refuses_a_clash_and_moves_nothing` |
| 3 | data safety | a rename across devices fails silently | medium | fallback to copy then remove, each error surfaced |
| 4 | data safety | the CLI could adopt from the vault into itself when `notes_path` already points there | low | the CLI compares the canonical paths and reports "already live in the vault"; `adopt` itself refuses the same or a nested folder (`adopt_refuses_the_same_or_a_nested_folder`) |
| 5 | correctness | the service creates `<vault>/notes` (and its `.deleted/`) at start, so a fresh vault shows an empty `notes` folder | ok | the ticket wants an ordinary folder; the scene shows it |
| 6 | correctness | two watchers over one folder would fire twice | ok | the notes watch is skipped when the root lies inside the vault |
| 7 | concurrency | the CLI moves files while the service watches the vault | ok | the watcher only emits `DataChanged`; the index syncs on the next burst |
| 8 | prose | the README paragraph, the installer line, the CLI usage and messages | ok | zero em dashes |

- Post-implementation CodeGraph: `NotesManager` is built in `core.rs` alone and read by
  the six tools; `adopt` has one caller, the CLI; `start_data_watcher` one caller, the
  server's `main`.

## Phase 4: Validate

- Tests run (commands and output): the gate ran every test; the focused runs above; on
  the box, read-only and dry: `rusty-cli notes path` printed the vault's `notes` folder
  (no `notes_path` setting on this box), `rusty-cli notes adopt --dry-run` listed the
  five notes and the two `.deleted/` files it would move and moved nothing (the folder
  still holds 7 files; the vault has no `notes/` folder from the CLI), and an unknown
  subcommand exits 2 with the usage.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: `GATE GREEN [diff]`, `receipt written:
  .git/rusty-gate-receipt`.
- Smoke evidence: the `reading` scene against the scratch vault shows `notes` as an
  ordinary folder in the tree with a zero count (the scratch service created it at start).
- Skips or pre-existing failures: the adoption was not run on Chad's notes (his one-shot
  to run, deliberately); the `/note` skill's new fallback is verified by reading.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (three adoption tests); REQ-002 PASS (`core.rs` by
  reading; `notes path` on the box); REQ-003 PASS (the index test); REQ-004 PASS (the
  first test's README and second run); REQ-005 PASS (the scene); REQ-006 PASS (the README
  section, the installer line).
- Docs: `README.md` (a Notes section), `docs/architecture.md` (the CLI bullet),
  `ROADMAP.md` (M8 line), the store's `/note` skill.
- Wiki: `update` run `2203d6aa` through the lifecycle: a notes claim added on
  `mcp-back-end.md`, the bridge claim's `lib.rs` evidence refreshed, the notes line in
  the tool list rewritten; `openwiki_finish` returned `status: complete`; the PostToolUse
  hook stayed silent and the genuine result was fed to `record-pipeline-tool-use.sh`.
- AAR: `docs/planning/knowledge/aar/AAR-014-notes-into-the-vault.md`; no new register IDs.
- Brain capture: timeline entry on `projects/rusty-v3` at delivery.
- Archive: this pair lives in `completed/`; the ticket in `closed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
