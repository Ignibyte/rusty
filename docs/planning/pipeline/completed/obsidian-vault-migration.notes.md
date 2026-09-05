---
title: Obsidian vault migration — notes
pipeline: bd3dd522-2b7b-4d64-ab22-2c7148c62a14
ticket: TICKET-026
---

# Obsidian vault migration: notes

## Recall (2026-09-05)

- Bulletins: bulletin 2 shapes validation; the tests build their own source vault under
  a temporary directory and never read Chad's.
- Register: `AD-rusty-vault-rules-001` (one wikilink style, `[[folder/slug]]`),
  `AD-rusty-bridge-retired-whole-001` (the vault's files and Obsidian's own settings
  stay; nothing writes Obsidian's config), `AD-rusty-bookmarks-in-state-001` (bookmarks
  are app state), `AD-rusty-mcp-only-back-end-001`.
- Wiki: `vault-and-brain.md` describes lenient pages, the migration (`rusty-cli brain
  migrate` rewrites bare links), the tag index; `mcp-back-end.md` the tool surface (80).
- Code read:
  - `migrate_vault`: `LinkIndex::build(&files)` then `rewrite_links(&text)` →
    `(rewritten, count, unresolved)`, body only, frontmatter untouched, `sync_all` and
    one commit; `LinkIndex` is private to `brain/mod.rs`, so the import's two methods
    live there and the pure parts in a sibling module.
  - `notes::adopt`: refuses clashes before moving anything, deletes nothing, leaves a
    README, `dry_run` reports; the tone to match.
  - `VaultManager`: `list_all_files` (pages under the root, dot-entries skipped),
    `write_page` (creates folders), `page_exists`, `exists`, `root`, `git_commit`,
    `flush_commits`; `ensure_dirs` makes the type folders and `inbox/` is one.
  - `rusty-mcp`: a tool is a `#[tool]` method with `Parameters<P>`; `mutate` emits
    `DataChanged`; `EXPECTED` lists every tool for `router_advertises_every_tool_once`.
  - `rusty-cli brain`: `parse_with_bools(rest, &["dry-run"])`, `fail(&e)`.
  - `Main.qml`: `ask(tool, args, kind)` and the `onResult` switch; `rootDialog` is a
    `FolderDialog`; `addBookmark`/`bookmarkIndex` keep `ui.bookmarks`.
  - Obsidian's `bookmarks.json`: `{"items":[…]}` with `type` file (`path` with `.md`,
    optional `subpath` `#Heading`, optional `title`), folder, search (`query`), group
    (`items`), url, graph.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-core/src/brain/import.rs` | new: `ImportPlan`, `ImportReport`, `ImportedBookmark`, `Scan`; `scan_vault`, `parse_bookmarks`, `report_page`; tests |
| `crates/rusty-core/src/brain/mod.rs` | `pub mod import`; `LinkIndex`, `build`, `rewrite_links` `pub(crate)`; `import_plan`, `import_vault` (the rollback) |
| `crates/rusty-mcp/src/main.rs` | `ImportParams`, `brain_import_plan`, `brain_import`; `EXPECTED` |
| `crates/rusty-cli/src/main.rs` | `brain import <vault> [--dry-run]` and its help line |
| `crates/rusty-app/qml/Main.qml` | the palette command, `importPicker`, `planImport`, `runImport`, `mergeBookmarks`, `importDetails`, the `importDialog`, two `onResult` cases, the `import:` scene |
| `scripts/screenshot.sh` | a seeded Obsidian vault under the scratch |
| `CLAUDE.md`, `AGENTS.md`, `README.md`, `docs/architecture.md` | 80 → 82 tools |

### The plan and the import

`scan_vault(source)`: every entry under the source, dot-entries skipped (`.obsidian`,
`.trash`, `.git` among them), `.md` files as pages (slug = the path without `.md`),
everything else as attachments. `import_plan`: the scan; a collision is a page slug
`page_exists` or an attachment path that `exists`; the `LinkIndex` over the brain's
pages and the incoming pages (their source paths) rewrites each incoming body without
writing and collects the unresolved targets; tags are the frontmatter list and the
inline `#tags` of the incoming pages, distinct as first seen; bookmarks are parsed and
kept when their target is an incoming page, a page in the brain, or a folder that will
exist, else listed as not carried. `import_vault`: the plan; then pages (the prefix as
it was, the rewritten body), attachments (`create_dir_all`, `copy`), the report page;
every created path recorded; on an error every recorded file is removed, `sync_all`
runs, and the error comes back saying nothing of the import remains; on success
`sync_all`, one commit, `flush_commits`.

### The app

"Vault: Import an Obsidian vault…" opens a `FolderDialog`; the chosen folder opens
`importDialog` and asks `brain_import_plan`; the dialog shows the summary line and the
details (collisions, skipped, unresolved, bookmarks); Import asks `brain_import`; the
answer's bookmarks are merged into `ui.bookmarks` through `bookmarkIndex`, the tree and
tags refresh, the dialog shows the result and the report's slug.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `import_plan_reports_what_it_found`; `scan_vault_skips_dot_entries`; the `import:` scene |
| REQ-002 | `import_vault_brings_pages_attachments_and_a_report`: `search` finds the page, `tags()` holds the tags, the link rewritten to the vault path, the attachment's bytes equal |
| REQ-003 | the plan test: the colliding slug in `collisions`, out of `pages`; after the import the brain's page unchanged |
| REQ-004 | `parse_bookmarks_flattens_groups_and_maps_kinds`; the plan test's kept and skipped bookmarks; reading of `mergeBookmarks` |
| REQ-005 | the import test: the `inbox/import-…` page with its sections; the CLI prints the same |
| REQ-006 | `import_vault_rolls_back_when_a_write_fails` (a file where a folder must go); the import test's byte-for-byte snapshot of the source |

### Risks

- **Data safety.** The source is opened read-only in every path; the brain gains files
  only under names that did not exist; the rollback removes only what this run created.
- **Names.** Obsidian file names with spaces and capitals are kept; `resolve_path`
  refuses `..`, which the scan never produces.
- **Big vaults.** One `LinkIndex` over everything, one `sync_all`: the migration's cost,
  once.
- **Keyboard.** The palette command, the dialog's buttons by Tab.
- **Theme.** The dialog uses the controls' own look and `termFont` for the details.

### CodeGraph

`codegraph_explore` over `migrate_vault`, `LinkIndex`, `sync_all`, `adopt`: the migration
path is one caller (`rusty-cli`), `sync_all` has eight; the import adds two callers of
`sync_all` and the first outside `migrate_vault` of `LinkIndex`. A second pass after
implementation goes in the ledger.

## Phase 3: Implement

As the manifest said. `brain/import.rs` (new): `Scan`, `ImportedBookmark`, `ImportPlan`,
`ImportReport`; `scan_vault`, `folders_of`, `parse_bookmarks`, `report_page`; three
tests. `brain/mod.rs`: `pub mod import`, `LinkIndex`, `build` and `rewrite_links`
`pub(crate)`, `import_plan`, `import_vault` with the rollback, three tests on a fixture
vault (`obsidian_vault`, `snapshot`). `rusty-mcp`: `ImportParams`, `brain_import_plan`,
`brain_import`, `EXPECTED`. `rusty-cli`: `brain import <vault> [--dry-run]`. `Main.qml`:
the command, `importPicker`, `planImport`, `runImport`, `mergeBookmarks`, `importDetails`,
`importDialog`, two `onResult` cases, the `import:` scene. `scripts/screenshot.sh`: the
seeded vault. `CLAUDE.md`, `AGENTS.md`, `README.md`, `docs/architecture.md`: 82 tools.
`qmllint` exit 0, `bash -n` on the script, `cargo fmt --all` ran.

Deviations: three gate rounds before green — a raw string closed early on `"#Heading"`
(`r##"…"##`), two clippy `contains()` nits, and two expectations of mine (the fixture
folder's name is the vault's name; `[[Note A]]` at the root already was its vault path,
so one link was rewritten, not two). The report slug takes a suffix when the minute's
name exists (F2).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | data safety | a source that holds the brain (`HOME`) | rejected: `.rusty` is a dot-entry and skipped; the brain itself as the source is refused by name |
| F2 | data safety | two imports in one minute wrote the same `inbox/import-<stamp>-<name>` slug, the second report over the first | **confirmed**; a `-2`, `-3`… suffix while the slug exists |
| F3 | correctness | a page without frontmatter through `split_raw` | rejected: the error path takes the whole file as the body, and the prefix is empty |
| F4 | correctness | the `LinkIndex` in `import_vault` rebuilt from `plan.pages` and the source | rejected: the same set the plan used, one call apart, no write between |
| F5 | data safety | the rollback removes files, not the folders it made | accepted: an empty folder holds no page and the next import fills or ignores it; noted in the wiki |
| F6 | correctness | the plan's `source` is canonical, so `import_vault`'s joins land in the real folder | rejected (read) |
| F7 | correctness | `mergeBookmarks` builds the app's shapes (`file`/`folder` path, `search` query, `heading` path and heading) and `bookmarkIndex` dedups by the same keys | rejected (read) |
| F8 | correctness | after the import the tree, pages and tags refresh twice (`refreshData` and `DataChanged`) | accepted: idempotent reads |
| F9 | keyboard first | the palette command, Import and Cancel by Tab, Escape closes | no finding |
| F10 | prose | the CLI prints a search bookmark with an empty path before its query | accepted: one line per bookmark, readable |
| F11 | correctness | `walk` treats `Foo.MD` as an attachment | accepted: Obsidian writes `.md`; a case-insensitive suffix would rename the page |
| F12 | secrets | `.obsidian/` is read for `bookmarks.json` alone; nothing else of Obsidian's is opened | no finding |
| F13 | correctness | CodeGraph: `import_vault → import_plan → scan_vault`; `LinkIndex` has its first callers outside `migrate_vault`, `sync_all` two more; the tools and the CLI are the only other callers | the blast radius matches the manifest |
| F14 | prose | the tool descriptions, the dialog's sentence, the report page | read against `no-ai-slop` |

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: red three times (the raw string, two clippy
  nits, two expectations), then `GATE GREEN [fast]` with the six new tests passing:
  `scan_vault_skips_dot_entries`, `parse_bookmarks_flattens_groups_and_maps_kinds`,
  `report_page_names_everything`, `import_plan_reports_what_it_found` (a collision left
  as it was, the tags, the unresolved link, four bookmarks kept and one skipped, the
  source byte for byte as before, the brain itself refused),
  `import_vault_brings_pages_attachments_and_a_report` (`[[Note B]]` → `[[sub/Note B]]`,
  the frontmatter as it was, the attachment's bytes, search, tags and links after the
  run, the report page, the source unchanged), `import_vault_rolls_back_when_a_write_fails`
  (a file where the attachments' folder must go: no page, no index row, no report).
- `cargo build -p rusty-app -p rusty-mcp` (23:56:20), then `SHOT_KEEP=1 scripts/screenshot.sh
  <scratch> "import:obsidian-vault"` with `RUSTY_SHOT_DELAY=5000`: the log clean; the
  picture shows the review dialog — the path, "Will bring in 2 pages in 2 folders, 1
  attachment, 2 tags and 4 bookmarks; 0 collisions skipped; 1 link unresolved. The vault
  is read and never written.", the details, Import and Cancel — REQ-001's scene. The kept
  scratch: the seeded vault's six files as seeded, no `inbox/` page — a plan writes
  nothing.
- REQ-002 to REQ-006 rest on the tests named in the regression table; REQ-001 on the plan
  test and the scene, with Chad's real vault to come.
- `bin/gate.sh --diff` after the last gated edit: green once on code without F2 (the
  patch had missed its anchor after `cargo fmt` rewrapped the line — caught by the
  patch's own assertion, not the gate), then after F2 landed: fmt, clippy, test, doc,
  shell-syntax, secrets, whitespace all ok, `receipt written: .git/rusty-gate-receipt`,
  `GATE GREEN [diff]`.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-006 satisfied — REQ-001 by the plan test and the
  scene (Chad's real vault to come), REQ-002 to REQ-006 by the tests the regression table
  names. None split, none waived.
- Wiki: two runs. `5a940cb9-2d78-4e02-9fee-e1e1bc4433fa` → `complete` with a warning:
  the vault page's claim had been anchored before F2 moved `import_vault`, so its sidecar
  was left unchanged; `15543bd8-ab1e-49e5-aefb-e9256526f12b` re-added the claim at the
  current range → `complete`. Prose: the vault page (a bullet, a failure mode, the
  tests), the back end (a bullet, the tests, two claims re-anchored, one added), the app
  (a bullet, the tests, the scenes claim extended, one added). `docs/architecture.md`,
  `CLAUDE.md`, `AGENTS.md` and `README.md` say 82 tools. The PostToolUse hook did not
  fire (tenth sighting); bulletin 3's recovery with the pair under `active/`, then
  `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-import-keeps-paths-and-refuses-001` in the AAR and
  the register. Brain: timeline entry on `projects/rusty-v3`.
