---
title: Favorites: notes
pipeline_id: 44e4d03a-c884-487c-a38f-f8120dd6bb1a
---

# Favorites: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices). Register: `AD-rusty-bookmarks-in-state-001` (bookmarks
  live under `bookmarks` in the workspace state; a tool is a seam for later),
  `PR-rusty-qml-component-scope-001`. TICKET-005 notes: the Bookmarks tab, the palette
  commands, the explorer's right-click. Code read: `Main.qml` already has `bookmarkList`,
  `isBookmarkedPath`, `addBookmark` (a toggle), `bookmarkCurrentPage`, `openBookmark`;
  `NoteTab` already takes `bookmarked` and raises `requestBookmark`; `Explorer` renders
  `rows` in a `ListView` and raises `requestBookmark(row)`; `QuickSwitcher.refilter`
  scores title and slug and keeps the top fifty; Ctrl+D is unbound.
- Decisions: the three locked decisions in the spec.

## Phase 2: Design

- Architecture and data flow: nothing new is stored. The window's `bookmarkList` is the
  source; `Explorer` and `QuickSwitcher` take a `favorites` property derived from it
  (the file and folder entries; the switcher takes the file slugs), and the note's
  `bookmarked` property already reflects `isBookmarkedPath`. The note's header gains a
  star that raises the existing `requestBookmark`; the menu entry is reworded. Ctrl+D
  calls `bookmarkCurrentPage`, which toggles. The explorer's Favorites section raises
  `openFavorite` (the window's `openBookmark`) and `removeFavorite` (the window's
  `addBookmark`, a toggle). The switcher lists favorites first on an empty query and stars
  them.
- File manifest:
  - `crates/rusty-app/qml/NoteTab.qml`: the star beside the reading toggle; the menu text.
  - `crates/rusty-app/qml/Main.qml`: the Ctrl+D `Shortcut`, the palette command's key, the
    two instances wired.
  - `crates/rusty-app/qml/Explorer.qml`: `favorites`, two signals, the section above the
    tree.
  - `crates/rusty-app/qml/QuickSwitcher.qml`: `favorites`, the ordering, the star.
  - Phase 5: `README.md`, `ROADMAP.md`, the wiki page `workspace-app.md`.
- Store consequences: none; the `bookmarks` key and its entries are unchanged.
- Tool contract: none.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | the `reading` scene against the scratch state (Orbit is bookmarked there): a lit star in the header |
  | REQ-002 | the `Shortcut` and the command's key by reading; the Hotkeys row in the `view:settings` scene |
  | REQ-003 | the `reading` scene: the Favorites section above the tree with the file and the folder from the scratch state |
  | REQ-004 | the `switcher` scene: the bookmarked page first and starred with an empty query (the design amends the ticket's "pure function" test: QML has no unit harness here, the scene against the known scratch state stands for it) |
  | REQ-005 | the `left:bookmarks` scene unchanged (four entries: file, folder, search, heading); no change to `write()` |
- Risks: data safety, none. Keyboard: Ctrl+D was unbound; it stands down while a terminal
  has focus. Theme: gold for the star, the tokens for the rest. Empty state: the section
  hides when there are no favorites. No back end: unchanged.
- CodeGraph evidence: no Rust symbol changes; the QML functions involved (`addBookmark`,
  `openBookmark`, `isBookmarkedPath`, `bookmarkCurrentPage`) are read by hand.

## Phase 3: Implement

- Built: the manifest as designed. `NoteTab.qml` (the star beside the reading toggle,
  gold when lit, a tooltip naming Ctrl+D; the menu entry reworded), `Main.qml` (the
  Ctrl+D `Shortcut` gated on a current note, the palette command renamed "Favorites: Add
  or remove the current file" with its key, `favorites` passed to the explorer and the
  switcher, `openFavorite` and `removeFavorite` wired to `openBookmark` and `addBookmark`),
  `Explorer.qml` (the Favorites section between the nav header and the tree: a
  micro-label, one row per file or folder with a star, an icon and the title, a rule
  beneath; left click opens, right click removes; hidden when empty), `QuickSwitcher.qml`
  (`favorites`, `isFavorite`, favorites first on an empty query, a star before the title).
- Deviations: none from the manifest. The palette command's name changed from
  "Bookmarks: Bookmark the current file" to the favorites wording so the Hotkeys table
  reads as the feature does; the Bookmarks tab keeps its name.
- Fast gate: `cargo build --workspace` clean (14.48 s); `bin/gate.sh --fast` on
  2026-09-03: `GATE GREEN [fast]`.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | a removed favorite whose page is open leaves the star unlit at once, since `bookmarked` binds to `isBookmarkedPath` over `bookmarkList` | ok | by reading |
| 2 | correctness | a favorite made from the explorer's right-click ("Bookmark…") lands in the section, since the section reads the same list | ok | by reading |
| 3 | keyboard | Ctrl+D stands down while a terminal has focus and when no note is current; the palette carries the action | ok | by reading |
| 4 | theme | the star is `gold`, the rows use `hover`, `muted`, `foreground`, `line` | ok | by reading |
| 5 | empty state | the section hides with no favorites; the Bookmarks tab's empty text still explains how to add one | ok | the `left:bookmarks` scene |
| 6 | scope | headings and searches stay out of the section and in the Bookmarks tab; the storage is untouched | ok | by reading `addBookmark` and `write()` |
| 7 | prose | the tooltip, the menu entries, the command name, the section label | ok | zero em dashes |

- Post-implementation CodeGraph: no Rust symbol changed; the QML paths are the window's
  bookmark functions, read by hand.

## Phase 4: Validate

- Tests run (commands and output): the gate ran every Rust test (unchanged); the
  screenshot script wrote `reading`, `switcher` and `left:bookmarks` against the scratch
  state with no QML error; the ten docs scenes wrote again, since the explorer shows in
  every one.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: `GATE GREEN [diff]`, `receipt written:
  .git/rusty-gate-receipt`.
- Smoke evidence: the `reading` scene shows the lit star beside "[ READ ]" and a
  Favorites section with "Orbit" (file) and "concepts" (folder) above the tree; the
  `switcher` scene shows "Orbit" first with a star on an empty query; the
  `left:bookmarks` scene shows the same four bookmarks as before (file, folder, search,
  heading). No synthetic input on the desktop.
- Skips or pre-existing failures: the click and right-click handlers, the key and the
  state's shape are verified by reading; the ticket's "pure function" test for the
  switcher's ordering is replaced by the scene against a known state, recorded in the
  design.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the `reading` scene, the binding by reading); REQ-002
  PASS (the `Shortcut` and the command by reading); REQ-003 PASS (the `reading` scene);
  REQ-004 PASS (the `switcher` scene against the scratch state, the ordering by reading);
  REQ-005 PASS (the `left:bookmarks` scene; `write()` untouched).
- Docs: `README.md` (the favorites sentences), `ROADMAP.md` (M8 line), the ten docs
  screenshots re-rendered (the explorer shows in every one).
- Wiki: `update` run `e0b0e717` through the lifecycle: the bookmarks claim on
  `workspace-app.md` refreshed, a favorites claim added, the bookmarks bullet extended;
  `openwiki_finish` returned `status: complete`; the PostToolUse hook stayed silent and
  the genuine result was fed to `record-pipeline-tool-use.sh` (the bulletin's path).
- AAR: `docs/planning/knowledge/aar/AAR-013-favorites.md`; no new register IDs.
- Brain capture: timeline entry on `projects/rusty-v3` at delivery.
- Archive: this pair lives in `completed/`; the ticket in `closed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
