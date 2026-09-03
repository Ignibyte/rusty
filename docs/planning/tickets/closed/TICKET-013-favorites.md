---
title: TICKET-013-favorites
status: done
ticket_number: 013
type: feature
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/favorites.spec.md
---

# TICKET-013-favorites

## Summary

Favorites are the bookmarks that already exist, made visible: a star on the note header, a
hotkey, a Favorites section at the top of the file explorer, and favorites first in the
quick switcher.

## Why

TICKET-005 landed bookmarks for files, folders, searches and headings, kept in the workspace
state, reachable from a right-click in the explorer, a palette command and the Bookmarks tab
of the left sidebar. Chad did not find them. The feature is there; the affordance is not.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a page is open, its header shall show a star that toggles the page's bookmark and reflects its state. | screenshot; QML test |
| REQ-002 | WHEN Ctrl+D is pressed on a page tab, the page's bookmark shall toggle. | hotkey test |
| REQ-003 | WHEN the file explorer is shown and bookmarks exist, a Favorites section shall sit above the tree listing the bookmarked files and folders, opening each on click and removing on right-click. | screenshot |
| REQ-004 | WHEN the quick switcher opens with an empty query, favorites shall list first, starred. | test on the switcher's ordering |
| REQ-005 | WHEN favorites are used, the Bookmarks tab shall keep searches and headings and the storage shape shall not change. | state-file test |

## Scope

- In: the star, the hotkey, the explorer section, the switcher ordering.
- Out: reordering favorites by drag; syncing favorites between machines.

## Notes

- Pipeline spec: docs/planning/pipeline/completed/favorites.spec.md
- Related docs: `crates/rusty-app/qml/BookmarksPane.qml`, `Explorer.qml`, `QuickSwitcher.qml`, `NoteTab.qml`; TICKET-005.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "we should have a way to favorite some documents".
- Follow-ups opened: none.
