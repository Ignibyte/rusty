---
title: Search operators, bookmarks and hotkeys: notes
pipeline_id: 833ac963-574a-478e-9790-ae31bcaccb8c
---

# Search operators, bookmarks and hotkeys: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-tags-one-index-001` (`tag:` terms are part of
  `brain_search`, not a tool), `AD-rusty-workspace-is-obsidian-001`,
  `PR-rusty-workspace-state-in-json-001`, `PR-rusty-qml-component-scope-001`,
  `PR-rusty-qml-signal-names-001`; completed notes of TICKET-003 (`split_tag_terms`,
  `pages_with_tags`, `pages_in`) and TICKET-004 (state under a key, scenes); wiki
  `workspace-app.md` (the left sidebar, `commandList()`, the state file),
  `vault-and-brain.md` (`brain_fts` holds slug, type and text; `brain_pages` the
  title and type); the brain's `projects/rusty-v3` timeline. The left sidebar already
  shows a dimmed Bookmarks tab.
- Decisions: the four in the spec.
- Seal: Chad's goal of 2026-09-02.

## Phase 2: Design

- Architecture and data flow: `parse_query(query) -> ParsedQuery { words, tags,
  paths, files, types, and the excluded of each }` with a quote-aware tokenizer.
  `SearchOptions { limit, page_type, case_sensitive, regex }`; `search_with` and
  `search_hybrid_with` take it, and the old `search` and `search_hybrid` call them with
  the defaults. `allowed_pages(parsed)` builds the set of slugs the operators admit
  (tags through `brain_tags`, the rest over `list_pages`), `None` when there are no
  operators. Plain mode: FTS as today, filtered by the set. Match case: the FTS hits
  are kept when the text holds every word as typed, with a snippet built around the
  first occurrence. Regex: the admitted pages' text is scanned with the pattern; hits
  rank by match count with a snippet around the first match. Hybrid applies the set to
  both halves and hands regex to the text search. The tool passes `case_sensitive` and
  `regex` through. The app: the search pane owns the two toggles and sends them with
  the query; `BookmarksPane` lists the window's `bookmarks` (a JSON array under the
  state's `bookmarks` key) and opens each by kind; the page menu, explorer menu, search
  pane, outline row menu and palette add them; Settings renders the command list.
- File manifest:

| File | Purpose |
|---|---|
| `crates/rusty-core/src/brain/mod.rs` | `ParsedQuery`, `parse_query`, `SearchOptions`, `search_with`, `search_hybrid_with`, `allowed_pages`, the two modes, tests |
| `crates/rusty-core/Cargo.toml` | `regex` |
| `crates/rusty-mcp/src/main.rs` | `BrainSearchParams` fields and description; smoke |
| `crates/rusty-app/qml/SearchPane.qml` | toggles, hint, bookmark action |
| `crates/rusty-app/qml/BookmarksPane.qml` | the pane |
| `crates/rusty-app/qml/Main.qml` | `bookmarks` state and functions, the sidebar tab, palette commands, scenes |
| `crates/rusty-app/qml/NoteTab.qml`, `Explorer.qml`, `RightPane.qml` | entry points |
| `crates/rusty-app/qml/SettingsPage.qml` | the Hotkeys table |
| `crates/rusty-app/build.rs` | the new QML file |
| `scripts/screenshot.sh` | seeded bookmarks, `left:bookmarks`, `view:settings` scenes |
| `README.md`, `ROADMAP.md`, `openwiki/` | the tier |

- Store consequences: none.
- Tool contract: `brain_search` gains `case_sensitive` and `regex` (optional booleans);
  the result shape is unchanged.
- Regression plan: REQ-001 and REQ-002 core tests (each operator, exclusion, quoted
  values, operators alone, match case, regex with a snippet), smoke; REQ-003 to REQ-005
  screenshots and reading; REQ-006 the router test and smoke.
- Risks: a regex over every page is linear in the vault's text (fine for hundreds of
  pages; the limit stops the scan early when there are no operators to apply first);
  the hint must not crowd the pane; bookmarks of a renamed page go stale (a rename
  rewrites links, not bookmarks: named as a limitation).
- CodeGraph evidence: `split_tag_terms` (two callers: `search`, `search_hybrid`),
  `pages_in` and `pages_with_tags` (the same two).

## Phase 3: Implement

- Built: the manifest. Core: `SearchOptions`, `ParsedQuery` with `has_operators`,
  `parse_query` over a quote-aware `query_tokens`, `search_with` (operators, then FTS5,
  match case through `keep_case`, regex through `search_regex`, operator terms alone
  through `pages_in`), `allowed_pages`, `snippet_around`, `search_hybrid_with` (the
  same operators on both halves, the text modes handed to `search_with`), and the old
  `search` and `search_hybrid` as wrappers; `regex` in `Cargo.toml`; the test
  `search_operators_and_modes` and the parser assertion in the tags test. Tool:
  `case_sensitive` and `regex` on `BrainSearchParams`, the description naming the
  operators, a regex call in the smoke test. App: `BookmarksPane.qml`; the `bookmarks`
  key of the workspace state with `bookmarkList`, `addBookmark` (a toggle),
  `removeBookmark`, `retitleBookmark`, `openBookmark`; the Bookmarks sidebar tab; the
  search pane's `Aa`, `.*` and bookmark chips and its operator hint; "Bookmark…" in
  the page menu and the explorer's row menu, "Bookmark heading" on an outline row,
  three palette commands; `scrollToHeadingText` with a pending heading in `NoteTab`;
  `revealFolder` in the explorer; the Hotkeys table in Settings from the palette's
  command list; a `folder` icon; the `view:` scene and seeded bookmarks in the
  screenshot script.
- Deviations: none from the manifest.
- Fast gate: `cargo test -p rusty-core -- search_operators_and_modes
  tags_index_search_and_properties` 2 passed; `cargo test -p rusty-mcp` 3 + 1 passed;
  `cargo build --workspace` clean.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | the regex rows were collected as the tail expression of a block that owned the statement and the connection guard, so the iterator outlived them | low | fixed: collect into a named `Vec` inside the scope (PR-rusty-collect-inside-scope-001) |
| 2 | SQLite lock | `keep_case` holds one guard across its row reads; `allowed_pages` takes its guards one call at a time; `search_regex` reads every row before matching | ok | by reading |
| 3 | correctness | a pattern that matches the empty string (`a*`) counts a match at every position; the count is bounded by the text length and only orders the results | note | accepted |
| 4 | keyboard | no new keys; the chips are pointer targets and the palette carries the bookmark commands; the hotkeys filter field is a plain `TextField` | ok | by reading |
| 5 | theme | chips use the active, hover and muted tokens; key chips the surface and line tokens; the hint the faint token | ok | screenshots |
| 6 | data safety | a bookmark keeps the path it was made with; a rename or delete leaves it stale | low | accepted; named in the wiki's failure modes |
| 7 | performance | the Settings page's `commands` binding re-evaluates `commandList()` on state changes; a few dozen closures | ok | accepted |
| 8 | prose | the hint, the tool description, the docs and the wiki against `no-ai-slop` | ok | clean |

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-core`: the operator test covers
  `path:`, `type:`, `file:`, `-path:`, `tag:`, `-tag:`, a quoted value, an unknown
  type, the parser's fields, match case with its snippet, regex with its snippet, regex
  with an operator, and a bad pattern. `cargo test -p rusty-mcp`: 3 passed; the smoke
  test's `path:concepts pl.n` with `regex` finds the page. The gate ran the whole
  workspace.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff] (receipt 2026-09-03T05:46:00Z).
- Smoke evidence: `scripts/screenshot.sh` scenes `left:bookmarks` (four bookmarks of
  the four kinds from the seeded state), `search:tag:theme path:concepts` (four themes,
  the chips), `left:search` (the hint), `view:settings` (the Hotkeys table with its
  filter); the first is `docs/screenshots/bookmarks.png`.
- Skips or pre-existing failures: the chips, the menus and the bookmark opening are
  verified by reading; no synthetic input on the desktop.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (core test, smoke); REQ-002 PASS (core test);
  REQ-003 PASS (two screenshots); REQ-004 PASS (screenshot from seeded state, the
  entry points and the save by reading); REQ-005 PASS (screenshot); REQ-006 PASS
  (router test, smoke).
- Wiki: `update` run through the lifecycle, openwiki_finish returned status complete (receipt 2026-09-03T05:47:28Z); `vault-and-brain`,
  `mcp-back-end` and `workspace-app` reconciled.
- Docs: README (search, bookmarks, hotkeys, the screenshot), ROADMAP (M8 line),
  `docs/architecture.md` (the pane), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-005-search-bookmarks-hotkeys.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | a block's tail expression borrowed the locals the block owned | PR-rusty-collect-inside-scope-001 |
| 2026-09-03 | one query parser for every search path | AD-rusty-search-operators-in-core-001 |
| 2026-09-03 | bookmarks in the workspace state | AD-rusty-bookmarks-in-state-001 |
