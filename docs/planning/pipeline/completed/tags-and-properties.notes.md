---
title: Tags and properties: notes
pipeline_id: 6d2a9c4e-7b31-4f58-a0e2-9c1d5e7f3b21
---

# Tags and properties: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-lenient-pages-001`, `AD-rusty-renderer-in-core-001`,
  `PR-rusty-scope-the-sqlite-guard-001`, `PR-rusty-qml-component-scope-001`,
  `PR-rusty-workspace-state-in-json-001`; completed notes of TICKET-002 (the properties
  block, the `rusty:tag/` links, the search pane) and TICKET-007 (the wiki update at
  Phase 5); wiki pages `vault-and-brain.md` (the index, `brain_tags` from frontmatter
  only today, `sync_tags`) and `workspace-app.md`; the brain's `projects/rusty-v3`
  timeline. Code: `sync_tags` writes frontmatter tags; the renderer already turns
  inline `#tags` into `rusty:tag/<tag>` links; `frontmatter::properties_of` returns
  the ordered mapping; `NoteTab.qml` shows properties read only.
- Decisions: the five in the spec.
- Seal: Chad's goal of 2026-09-02 covers tickets 2 to 6.

## Phase 2: Design

- Architecture and data flow: `links::tags(text)` scans the body outside code for
  `#tags` (the renderer's rule, shared); `index_page` and `sync_page` write
  `brain_tags` from the frontmatter list plus the inline set, deduplicated without
  case. `BrainManager::tags()` groups `brain_tags` by tag with counts and adds parent
  counts for nested tags. `search` and `search_hybrid` parse `tag:` terms out of the
  query: pages carrying the tag or a nested one form the allowed set; with no other
  words the result is that set newest first, else the FTS (or fused) result filtered to
  it. `set_property` and `remove_property` parse the frontmatter YAML as an ordered
  mapping, change one key, re-serialise, and write the original body after it;
  `sync_page` then re-indexes. Three tools: `brain_tags`, `brain_set_property`,
  `brain_remove_property`. The app: `RightPane` gains a Tags tab (tree from the counts);
  `NoteTab` turns the properties block into an editor that calls the two tools and
  re-renders on `dataChanged` (the tools mutate); `rusty:tag/` and the pane search
  `tag:<name>` in the search pane.
- File manifest:

| File | Purpose |
|---|---|
| `crates/rusty-core/src/brain/links.rs` | `tags(text)` inline scanner |
| `crates/rusty-core/src/brain/frontmatter.rs` | `set_property`, `remove_property` on raw text |
| `crates/rusty-core/src/brain/mod.rs` | tags in the index, `tags()`, `tag:` in search, the two property methods |
| `crates/rusty-mcp/src/main.rs` | the three tools, router list; smoke test |
| `crates/rusty-app/qml/RightPane.qml` | the Tags pane |
| `crates/rusty-app/qml/NoteTab.qml` | the properties editor |
| `crates/rusty-app/qml/Main.qml` | `rusty:tag/` and pane clicks to `tag:` searches; `brain_tags` refresh |
| `scripts/screenshot.sh` | fixture tags and a `right:tags` scene |
| `README.md`, `ROADMAP.md`, `openwiki/` | the tier |

- Store consequences: none; `brain_tags` already has `(slug, tag)`.
- Tool contract: `brain_tags` (none) returns `[{tag, count}]`; `brain_set_property`
  (`slug`, `key`, `value` JSON) and `brain_remove_property` (`slug`, `key`) return the
  page; `brain_search` keeps its shape and learns `tag:`.
- Regression plan: REQ-001 scanner and index tests; REQ-002 `tags()` and search tests,
  smoke; REQ-003 screenshot of the pane, the click path by reading; REQ-004 round-trip
  tests (order kept, body identical); REQ-005 screenshots of the editor; REQ-006 by
  reading (`dataChanged` re-render).
- Risks: a property edit on a page with unreadable YAML (refused with the error); the
  single connection (each method scopes its guard); nested tags with a trailing slash.
- CodeGraph evidence: `sync_tags` (2 callers in `mod.rs`), `search` (3 callers:
  `search_hybrid`, the tool, the CLI), `properties_of` (1 caller, `render_page`).

## Phase 3: Implement

- Built: the manifest. `links::tags` (Obsidian's rule, code skipped, case-insensitive
  dedupe); `sync_tags` takes the content and merges inline tags with the frontmatter
  list; `BrainManager::tags()` with parent counts; `split_tag_terms`,
  `pages_with_tags`, `pages_in` and `search_text` behind `search` and `search_hybrid`;
  `frontmatter::set_property` and `remove_property` over the ordered YAML mapping
  (`shift_remove` keeps the order), `BrainManager::set_property`, `remove_property`
  and `write_edited` (version, write, sync, commit; unchanged text is a no-op). Three
  tools, the router list, the smoke test. The Tags pane (`RightPane`), the properties
  editor (`NoteTab`: `PropertyRow`, chips, checkbox, typed fields, remove, add with a
  type), `brain_tags` in the window's refresh, `tag:` searches from pages, chips and the
  pane, two palette commands, fixture tags and a `right:tags` scene in the screenshot
  script.
- Deviations: none from the manifest.
- Fast gate: `cargo test -p rusty-core` 234 + 7 passed; `cargo test -p rusty-mcp` 3 + 1
  passed; clippy in the gate.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | `Mapping::remove` swaps the last key into the removed slot, breaking property order | medium | fixed (`shift_remove`), covered by the round-trip test |
| 2 | data safety | a property edit re-serialises the frontmatter with serde_yaml's style (quotes, indentation may change) while the body is byte for byte; Rusty's own writer uses the same style | note | accepted; the test pins the body |
| 3 | SQLite lock | `tags()` collects rows under one scoped guard and computes outside it; `pages_with_tags` holds one guard for its statements; `pages_in` uses `list_pages` (its own guard) | ok | verified by reading |
| 4 | correctness | the tag scanner and the renderer's `#tag` rule are two implementations of one rule | low | the scanner test and the renderer test pin the same samples; a shared function is a later tidy-up |
| 5 | keyboard | the properties editor: Enter commits a field, Escape leaves the add row, chips add on Enter; no new global keys | ok | by reading |
| 6 | theme | tag chips and the pane use the `tag` token; editors use `hover`, `line`, `faint` | ok | screenshots |
| 7 | prose | docs, tool descriptions and the wiki pages against `no-ai-slop` | ok | clean |

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-core`: 234 passed, 7 integration
  passed (`inline_tags_follow_obsidian_rules`, `property_edits_keep_order_and_body`,
  `tags_index_search_and_properties` among them). `cargo test -p rusty-mcp`: 3 passed;
  `tests/smoke.rs` 1 passed (an inline `#smoke/test` reaches `brain_tags`, `tag:smoke`
  finds the page, a property is set with the body intact and removed again).
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff]; fmt, clippy (-D warnings), tests (234 + 7 core, 3 + 1 mcp, 13 app), doc, shell-syntax, secrets, whitespace all ok; receipt at 05:07:33Z.
- Smoke evidence: `scripts/screenshot.sh` scenes `reading` (the properties editor with
  chips and per-row removal), `right:tags` (the Tags tree with counts) and
  `left:search,search:tag:launcher,right:tags` (one result for the tag); the last is
  `docs/screenshots/tags-and-properties.png`.
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (scanner and index tests); REQ-002 PASS (`tags()` and
  `tag:` search tests, smoke); REQ-003 PASS (screenshots; the click paths by reading);
  REQ-004 PASS (round-trip tests, smoke); REQ-005 PASS (screenshot of the editor; the
  tools behind it in the smoke test); REQ-006 PASS (by reading: a property result reloads
  the tab, and the count follows `properties.length`).
- Wiki: `update` run through the lifecycle, runs fddc9535 and a73ea4bc finished status complete (the second retracted 18 duplicate claims left by the first init batch and refreshed four drifted evidence ranges); receipt at 05:10:18Z; pages `vault-and-brain`,
  `mcp-back-end` and `workspace-app` reconciled with six new claims.
- Docs: README (the tier, the tools, the screenshot), ROADMAP (M8 line), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-003-tags-and-properties.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | `Mapping::remove` reorders keys | PR-rusty-yaml-mapping-shift-remove-001 |
| 2026-09-03 | one tag index for two sources; `tag:` inside search | AD-rusty-tags-one-index-001 |
