---
title: Favorites
pipeline_id: 44e4d03a-c884-487c-a38f-f8120dd6bb1a
status: Phase 5 — Complete PASS
ticket: TICKET-013
ticket_doc: docs/planning/tickets/open/TICKET-013-favorites.md
aar: docs/planning/knowledge/aar/AAR-013-favorites.md
sealed: not required (affordances over the bookmarks that exist; no new tab, store or dependency). Direction: Chad, 2026-09-03 15:40, "we should have a way to favorite some documents"; 16:00, "lets start working on these" (relayed by the rustal session)
created: 2026-09-03
---

# Favorites: spec

## Intent

Favorites are the bookmarks TICKET-005 landed, made visible where a person looks: a star
on the note header, Ctrl+D, a Favorites section at the top of the file explorer, and
favorites first in the quick switcher when the query is empty. The storage does not
change; the Bookmarks tab keeps searches and headings.

## Scope

- In: the star in the note header (state and toggle), the key and its palette command,
  the explorer's Favorites section (files and folders; click opens, right-click removes),
  the switcher's ordering with a star, docs and wiki.
- Out (named seams, not forgotten): reordering favorites by drag; syncing favorites
  between machines; a change to the bookmark JSON.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a page is open, its header shall show a star that toggles the page's bookmark and reflects its state. | screenshot; by reading the binding to `bookmarked` |
| REQ-002 | WHEN Ctrl+D is pressed on a page tab, the page's bookmark shall toggle. | the `Shortcut` and the palette command by reading; the Hotkeys table |
| REQ-003 | WHEN the file explorer is shown and bookmarks exist, a Favorites section shall sit above the tree listing the bookmarked files and folders, opening each on click and removing on right-click. | screenshot |
| REQ-004 | WHEN the quick switcher opens with an empty query, favorites shall list first, starred. | test on the switcher's ordering (a pure function) |
| REQ-005 | WHEN favorites are used, the Bookmarks tab shall keep searches and headings and the storage shape shall not change. | state-file test (the `bookmarks` key and its entries by reading; no Rust change) |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | A favorite is a bookmark of kind `file` or `folder`; nothing new is stored. | TICKET-005's storage already holds them; the feature was invisible, not missing. | A separate favorites list (two lists to keep in step). |
| 2 | The star sits in the note's view header beside the reading toggle, drawn from the window's `isBookmarkedPath`. | The header is where a person looks for the page's state; the window owns the list. | A star in the tab strip (one more thing in a tight row). |
| 3 | The explorer's section is a header above the tree, not rows in it. | The tree is the vault's real folders; favorites are the user's view of it. | Marking rows in place (no gathering). |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-013-favorites.md`
- Intake: none
- Design references: `crates/rusty-app/qml/BookmarksPane.qml`, `Explorer.qml`,
  `QuickSwitcher.qml`, `NoteTab.qml`, the bookmark functions in `Main.qml` (lines 141 to
  160); TICKET-005's spec (`AD-rusty-bookmarks-in-state-001`)
- Architecture: `docs/architecture.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
