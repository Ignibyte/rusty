---
title: Knowledge workspace shell: notes
pipeline_id: 6b0c5d3e-2c0f-4b7e-9f0a-1d2e3f4a5b6c
---

# Knowledge workspace shell: running notes

## Phase 1: Plan

- Recall: the app already has a Brain tab (type tree, search, page view with rendered
  markdown and clickable wikilinks, edit and save of the compiled truth, timeline append,
  links, open in Obsidian, capture) and a Notes tab (daily pages). The core has the vault
  manager (one folder level, the type table), frontmatter parsing with the `## Timeline`
  section, an FTS and link index, the semantic index, page versions, and the Obsidian
  bridge. Qt's `Text.MarkdownText` is what renders today; it has no callouts, embeds or
  footnotes, and QtWebEngine is installed on the box but is not the path (decision 1).
  Chad's three screenshots (the Tasks tab, Obsidian's graph, an Obsidian page) and the
  Replit mock are the visual references; the mock is in `docs/design/`.
- Decisions: the six in the spec.
- Seal: pending Chad.

## Phase 2: Design

- Seal: 2026-09-02, Chad's words on the spec; amendments recorded there (Obsidian's
  layout and keys, everything a tab, agents as tabs and as the right pane, pages without
  frontmatter, whole-file source editing with autosave).
- References read for the design: Chad's three screenshots (the Tasks tab today; Obsidian's
  graph view with the explorer, tab strip and the Filters/Groups/Display/Forces panel;
  an Obsidian page with the breadcrumb, inline title, Properties block, body, and the
  status bar "0 backlinks · 4 properties · 43 words · 402 characters"), and the Replit
  mock (agent as a pane, command layer, breadcrumb toolbar with a read toggle).

### Architecture and data flow

- `rusty-core` owns every vault rule. New module `brain::render`: Obsidian-flavoured
  markdown to the HTML subset Qt's rich text engine understands, with an outline, the
  link list, a task count and word/character counts. New module `brain::links`: one
  scanner for `[[target#heading|alias]]` and `![[embed]]` that the indexer, the renderer,
  the migration and the move rewrite all share. `VaultManager` learns the real folder
  tree (recursive walk, dot-folders skipped), folder creation, path renames and soft
  deletes into `archive/`. `BrainManager` gains `tree`, `render_page`, `write_raw`,
  `new_page`, `new_folder`, `delete_folder`, `rename` (page or folder, links rewritten
  vault-wide, index rows moved) and `unresolved`.
- `rusty-mcp` adds eight thin tools: `brain_tree`, `brain_render`, `brain_write_page`,
  `brain_new_page`, `brain_new_folder`, `brain_delete_folder`, `brain_rename`,
  `brain_unresolved`. Writes go through `mutate()`, so `list_changed` follows.
  `brain_get_links` keeps its shape and adds `context` (the line the link sits on) and
  resolved targets.
- `rusty-app` renders what the tools return. The reading view is a `Text` with
  `Text.RichText`; links carry a `rusty:` scheme (`rusty:page/<slug>`,
  `rusty:new/<name>`, `rusty:task/<n>`, `rusty:tag/<tag>`) that QML routes. The source
  editor is a `TextArea` whose `textDocument` is handed to a C++ `QSyntaxHighlighter`
  subclass registered in the QML module; it colours each line from spans computed by a
  Rust tokenizer (`src/markdown.rs`) through a cxx bridge, so the rules live in Rust and
  are unit-tested there. Colours come from `Theme`, which now also reads the Omarchy
  theme's `obsidian.css` tokens and Alacritty's ANSI palette.
- Tabs: one QML `ListModel` of `{kind, title, slug, session, program, cwd, pinned}` where
  kind is `page`, `terminal`, `tasks`, `memory`, `skills`, `secrets` or `settings`
  (`graph` and `search` follow in later tickets). Persisted through `Terminals.save`;
  older files without `kind` load as terminals.
- Screenshots for the record come from the app itself: `RUSTY_SHOT=<png>` grabs the
  window after it settles and quits, under `QT_QPA_PLATFORM=offscreen`, against a scratch
  vault served by a scratch `rusty-mcp` on another port. No real data, no workspace switch.

### File manifest

| File | Purpose |
|---|---|
| `crates/rusty-core/Cargo.toml` | `pulldown-cmark` 0.13, no default features |
| `crates/rusty-core/src/brain/frontmatter.rs` | lenient parse: no fences means empty frontmatter and the whole text as body; `title` and `type` default; `fill_defaults(slug)` |
| `crates/rusty-core/src/brain/links.rs` | `WikiRef`, `scan`, `rewrite_for_move`, `strip_target` |
| `crates/rusty-core/src/brain/render.rs` | `Style`, `Rendered`, `render`, callouts, embeds, tasks, highlights, tags, comments, footnotes, tables, code |
| `crates/rusty-core/src/brain/vault.rs` | `VaultNode`, `tree`, recursive `list_all_files`, `create_folder`, `rename_path`, `archive_path`, `type_for_folder` |
| `crates/rusty-core/src/brain/mod.rs` | the manager methods above; link indexing with context and resolution; `sync_page` on lenient pages |
| `crates/rusty-mcp/src/main.rs` | the eight tools, their parameter types, the router list |
| `crates/rusty-mcp/tests/smoke.rs` | tree, new page, render, rename on the scratch vault |
| `crates/rusty-app/build.rs` | new QML files, `cpp_file` for the highlighter, `src/markdown.rs` |
| `crates/rusty-app/cpp/highlighter.h`, `.cpp` | `MarkdownHighlighter`, a `QSyntaxHighlighter` on a `QQuickTextDocument`, `QML_ELEMENT` |
| `crates/rusty-app/src/markdown.rs` | line tokenizer and its cxx bridge |
| `crates/rusty-app/src/omarchy.rs` | `Tokens` from `obsidian.css` and Alacritty's palette; `RUSTY_OMARCHY_THEME_DIR` override |
| `crates/rusty-app/src/theme.rs` | `surface`, `surfaceAlt`, `line`, `muted`, `faint`, `tokens` (JSON), `shotPath` |
| `crates/rusty-app/src/terminals.rs` | tab JSON with `kind`, `slug`, `pinned` |
| `crates/rusty-app/qml/Main.qml` | the Obsidian layout: ribbon, left sidebar, tab strip and stack, right sidebar, status bar, overlays, keys, navigation history |
| `crates/rusty-app/qml/Explorer.qml` | the folder tree with counts, expand state, context menu, inline rename, new note and folder, move, delete |
| `crates/rusty-app/qml/SearchPane.qml` | full-text search with snippets |
| `crates/rusty-app/qml/NoteTab.qml` | one page: view header with breadcrumb and toggle, inline title, properties, reading view, source editor, autosave |
| `crates/rusty-app/qml/RightPane.qml` | Backlinks, Outgoing links, Outline, Agent |
| `crates/rusty-app/qml/AgentTerminal.qml` | the tmux-backed terminal, used by tabs and the right pane |
| `crates/rusty-app/qml/QuickSwitcher.qml`, `CommandPalette.qml` | Ctrl+O and Ctrl+P |
| `crates/rusty-app/qml/Icon.qml`, `icons.js` | inline SVG icons drawn in the theme's colours |
| removed: `BrainPage.qml`, `NotesPage.qml`, `PageView.qml` | replaced by the workspace |
| `README.md`, `ROADMAP.md`, `docs/architecture.md` | tool count, workspace section, as-built |

### Store consequences

- No schema change. `brain_links.context` (already a column) now carries the line the
  link sits on; `to_slug` holds the resolved slug when the target resolves (exact slug,
  unique basename, title or alias) and the raw target otherwise, which is what
  `unresolved` selects (`to_slug NOT IN brain_pages`).
- The vault format stays plain markdown Obsidian reads. Pages may live in any folder and
  may lack frontmatter; Rusty's writers still write frontmatter.
- A rename moves the file, rewrites links in other files (fenced code untouched), then
  moves index rows (`brain_pages`, `brain_fts`, `brain_links`, `brain_tags`,
  `brain_aliases`, `brain_timeline`, `brain_versions`, `brain_chunks`) by exact slug or
  by folder prefix, and commits once. Deletes stay soft (`archive/`).

### Tool contract

| Tool | Arguments | Returns |
|---|---|---|
| `brain_tree` | none | the root `VaultNode` (`name, path, kind, pages, children`) |
| `brain_render` | `slug`, optional `style` (colour names) | `html, outline[{level,text}], links[{target,slug,alias,embed}], unresolved[], tasks, words, characters, frontmatter` |
| `brain_write_page` | `slug`, `content` (whole file) | the page as re-read |
| `brain_new_page` | `folder`, optional `name` | `slug` (Untitled, Untitled 1, ... when no name) |
| `brain_new_folder` | `path` | `path` |
| `brain_delete_folder` | `path` | `archived` path |
| `brain_rename` | `from`, `to` (page or folder; a `to` ending in `/` means into that folder) | `{from, to, pages_rewritten}` |
| `brain_unresolved` | none | `[{from_slug, target, context}]` |

Nothing is renamed or removed; the 59 tools keep their names and shapes. Obsidian bridge
tools stay until TICKET-006.

### Regression plan

| REQ | Evidence |
|---|---|
| REQ-001 | core tests: nested tree with counts; rename rewrites `[[a/b]]`, `[[a/b\|x]]`, `[[a/b#h]]`, `![[a/b]]`, unique `[[b]]`, `](a/b.md)`, leaves fenced code; folder rename rewrites by prefix; index rows follow. Smoke: `brain_tree`, `brain_new_page`, `brain_rename` on the scratch vault |
| REQ-002 | offscreen screenshot with three tabs; tab JSON round trip test; keyboard walk in the log |
| REQ-003 | renderer tests per construct; offscreen screenshot of a fixture page |
| REQ-004 | `write_raw` round trip test (frontmatter and timeline unchanged); highlighter tokenizer tests; smoke |
| REQ-005 | index tests: backlinks carry the line, outgoing resolved, unresolved listed; screenshot of the right pane |
| REQ-006 | screenshots of the switcher and the palette; switcher create path uses `brain_new_page` (smoke) |
| REQ-007 | screenshots at 1280 and 1600 px in two themes via `RUSTY_OMARCHY_THEME_DIR` |
| REQ-008 | screenshot with a terminal tab and the Agent pane |
| REQ-009 | smoke of capture, daily, search and timeline append through the tools; the palette lists the commands |

### Risks

- Data safety: the rewrite touches many files; it runs on a scan that only matches the
  moved target's exact forms, skips fenced code, and commits before and after. Deletes
  are soft. Tests use scratch vaults only.
- The single SQLite connection: every new manager method scopes its guard; no method
  holds one across another manager call (`PR-rusty-scope-the-sqlite-guard-001`).
- Keys: the workspace keys are disabled whenever a terminal has focus.
- No back end: the workspace shows the connection notice and keeps the last tree.
- Theme: every colour is a token; the two-theme screenshots are the check.
- Editing races: a `dataChanged` while the editor is dirty does not reload; the page is
  re-read after the next save.

### CodeGraph evidence (`scripts/codegraph.sh explore`, 2026-09-02)

- `BrainManager`: 82 symbols across 4 files; 16 callers in enrichment, core,
  conversation_archive, process_manager and 6 more. Additive methods only; no caller
  changes its call.
- `VaultManager`: 2 callers (`brain/mod.rs`, its tests); `root` used by `mod.rs`.
  `list_all_files` (3 callers in `mod.rs`) changes meaning from one level to recursive;
  the callers (`sync_all`, `migrate_vault`) want every page, so this is the fix they need.
- `parse_page`: 14 callers in `frontmatter.rs` and `mod.rs`. Lenient parsing returns
  where an error was returned; every caller already handled `Ok`, and the strict error
  remains for malformed YAML.
- `parse_wiki_links`: 6 callers in `mod.rs`; replaced by `links::scan`, same
  deduplicated-target contract plus context.
- `LinkIndex`: 1 caller (`migrate_vault`); its resolver is reused by the indexer.
- App side: `PageView.qml` has two users (Brain and Notes pages), both removed with it.

Status: Phase 2 — Design PASS.

## Phase 3: Implement

- Built: the manifest as designed. Core: `brain::links` (one scanner, move rewrite),
  `brain::render` (pulldown-cmark 0.13 to Qt rich text: wikilinks, embeds with a depth
  limit, callouts through private-use markers, tasks as `rusty:task/n` links, tables,
  footnotes, highlights, tags, comments stripped, a heading marker the app splits on),
  lenient frontmatter with `fill_defaults`, the recursive vault tree and folder operations,
  `render_page`, `write_raw`, `new_page`, `new_folder`, `delete_folder`, `rename` (page and
  folder) with `move_index_rows`, `unresolved`, link rows with context and resolution
  refreshed on every sync. Server: eight tools, the watcher-driven `sync_all` in the
  indexer loop, the smoke test walking new page, whole-file write, render, rename, tree.
  App: `Theme` tokens from `obsidian.css` and the Alacritty palette, `MarkdownHighlighter`
  (C++ shim, Rust tokenizer with UTF-16 spans), `Tools.grabWindow`, tabs with a kind, the
  workspace state file, and the QML files in the manifest; `BrainPage`, `NotesPage` and
  `PageView` removed.
- Deviations: the workspace state (sidebar widths, panes, expanded folders, the pane's
  agent) lives in `~/.config/rusty/workspace.json` through `Terminals.loadState/saveState`
  rather than QtCore `Settings`, because `Settings` rewrote string properties with their
  defaults during the run (ints survived); `Settings` keeps only the window size and last
  tab. Screenshots use `QQuickWindow::grabWindow` from a C++ `Tools` object, because an
  item grab cannot start on the offscreen platform. Terminals start their session when
  first shown (`ready`), not at load, after the right pane's terminal launched an agent
  during a screenshot. `scripts/screenshot.sh` was added to the manifest as the way the
  record's screenshots are made.
- Fast gate: `cargo clippy --workspace --all-targets -- -D warnings` green on 2026-09-02
  after three rounds (derivable Default, `is_none_or`, `strip_prefix`); a GCC warning in
  cxx's generated `Vec` constructor is silenced in the build script alongside the existing
  Qt one.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | inline `Component { NoteTab { theme: theme } }` bound the tab's own property to itself; every tab loaded with `theme` undefined | high | fixed: the Rust objects carry distinct ids and the window exposes `theme`, `backend`, `terminals`; inline components bind through `win.` (PR-rusty-qml-component-scope-001) |
| 2 | correctness | `property bool left` on a `Splitter` component overrode `Item.left`, a final property; the whole window failed to load, silently, because Qt logs go to the journal when stderr is not a tty | high | fixed (renamed `isLeft`); the log path is written down (PR-rusty-qt-logs-in-journal-001) |
| 3 | correctness | the palette filtered before its command list was set; it showed "No matching command" | medium | fixed (`onCommandsChanged: refilter()`) |
| 4 | correctness | `rightPane: rightPane` inside the state object resolved to the sidebar item (ids win over the scope object's properties) and `JSON.stringify` threw on the cycle | medium | fixed (`ui.` prefix throughout the state object) |
| 5 | correctness | pulldown-cmark splits `[!kind]` into several text events, so the callout head was never seen | medium | fixed: the head is marked with private-use characters before parsing |
| 6 | correctness | code blocks passed through the inline text path (highlights and tags inside code) | medium | fixed: block text is collected raw |
| 7 | data safety | a rename normalises the moved page's frontmatter only when its title was the old file name; other pages change only where a link is rewritten, fenced code untouched; deletes soft | note | accepted |
| 8 | data safety | the right pane's terminal started `claude` at load during a screenshot run, in a scratch folder | medium | fixed: terminals start when shown; the script kills `rusty-shot-*` and `rusty-pane-*` sessions (PR-rusty-lazy-pane-terminals-001) |
| 9 | SQLite lock | every new manager method scopes its guard; `index_links` runs on the guard its caller holds; `DbResolver::resolve` takes and drops its own | ok | verified by reading |
| 10 | keyboard | workspace keys are disabled while a terminal has focus (`objectName == "term"`); the four terminal keys stay global | ok | verified by reading |
| 11 | theme | one literal red for the disconnected dot | low | fixed: the theme's `red` token |
| 12 | prose | docs and comments read against `no-ai-slop` | ok | clean |

- Post-implementation CodeGraph: the index predates this diff (49 files); the one-hop
  dependents of `BrainManager`, `VaultManager`, `parse_page` and `list_all_files` were
  checked by hand against the design's blast radius: every caller compiles and the tests
  pass, and `migrate_vault` and `sync_all` now see nested folders, which they wanted.
  `scripts/codegraph.sh index` at the next design phase picks up the new modules.

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-core`: 231 passed, 7 integration
  passed, 0 failed. `cargo test -p rusty-mcp`: unit 3 passed; `tests/smoke.rs` 1 passed
  (new page, whole-file write, render with a style, rename with one page rewritten, tree,
  no unresolved). `cargo test -p rusty-app`: 13 passed (tokenizer, tabs JSON, tokens).
- Gate run: `bin/gate.sh --diff` on 2026-09-03T04:37:05Z: GATE GREEN [diff]; fmt, clippy (-D warnings), tests (231 core + 7 integration, 3 + 1 smoke mcp, 13 app, cli), doc, shell-syntax, secrets (133 gated files), whitespace all ok; receipt fab8679 written.
- Smoke evidence: `scripts/screenshot.sh` against a scratch vault, offscreen: reading view
  (callout, tasks, highlight, tag, table, properties with list chips, backlinks with
  context, status bar counts), source editor with highlighting, quick switcher, command
  palette, agent pane running a shell beside the note, search with a bold snippet and the
  outline pane, the Tasks view as a tab, and the same page in catppuccin-latte at
  1280x800. The images are in `docs/screenshots/`; the workspace state file round-tripped
  the panes the scenes changed.
- Skips or pre-existing failures: pointer and keyboard walks in the running window are not
  automated (probes never drive the app with synthetic input); Chad's hands-on check stands
  in. `TasksPage.qml:97` reports a `Dialog` implicitWidth binding loop that predates this
  work.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (vault tests, rename tests, smoke); REQ-002 PASS
  (screenshots with four tabs, one pinned; tab JSON round trip in the app tests); REQ-003
  PASS (renderer tests per construct, reading screenshot); REQ-004 PASS (`write_raw` round
  trip test, tokenizer tests, source screenshot; autosave by reading); REQ-005 PASS (index
  tests for context, resolution and unresolved; right pane screenshots); REQ-006 PASS
  (switcher and palette screenshots; the create path is `brain_new_page`, covered by the
  smoke test); REQ-007 PASS (dark at 1500 and light at 1280 screenshots, every colour a
  token); REQ-008 PASS (terminal tab and agent pane screenshots); REQ-009 PASS (capture,
  daily note and timeline append are palette commands over the same tools; search is the
  left pane; the tools are exercised by the smoke test and the core tests).
- Docs: README (the workspace, vault tools, Obsidian), ROADMAP (M8 line), architecture (as
  built, vault rules), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-002-knowledge-workspace-shell.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-02 | inline components self-referenced `theme` | PR-rusty-qml-component-scope-001 |
| 2026-09-02 | a final Item property overridden; no error visible | PR-rusty-qt-logs-in-journal-001 |
| 2026-09-02 | QtCore Settings rewrote strings with defaults | PR-rusty-workspace-state-in-json-001 |
| 2026-09-02 | item grab cannot start offscreen | PR-rusty-offscreen-shots-grab-window-001 |
| 2026-09-02 | a pane terminal launched an agent at load | PR-rusty-lazy-pane-terminals-001 |
| 2026-09-02 | the renderer, the highlighter and the vault rules | AD-rusty-renderer-in-core-001, AD-rusty-workspace-is-obsidian-001 |
