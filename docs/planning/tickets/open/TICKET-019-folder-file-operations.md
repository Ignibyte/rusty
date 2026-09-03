---
title: TICKET-019-folder-file-operations
status: open
ticket_number: 019
type: feature
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/folder-file-operations.spec.md
---

# TICKET-019-folder-file-operations

## Summary

New file, new folder, rename, move by drag and delete under a folder root act on the disk; delete moves to the trash; editing a text file in its tab.

## Why

Part one (TICKET-016) reads the disk. The explorer is half a file explorer until it can change what it shows, and an agent's repository needs a rename or a delete now and then without leaving the app.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN new file, new folder, rename, move by drag and delete are used under a folder root, they shall act on the disk, and delete shall move to the trash (`gio trash` or the XDG trash spec). | tests on a temporary tree |
| REQ-002 | WHEN a text file's tab is edited, it shall save to the disk on the same autosave the note editor uses, and a file changed outside shall be reloaded on Refresh. | test on a temporary file; smoke |
| REQ-003 | WHEN a folder root is refreshed, the tree shall reflect the disk; a watcher on the roots is welcome but not required. | reading |

## Scope

- In: REQ-001 to REQ-003; the trash; the text editor for files outside the vault.
- Out: git operations; permissions; the vault's own files (the brain tools stay their path).

## Notes

- Pipeline spec: docs/planning/pipeline/active/folder-file-operations.spec.md
- Related docs: `crates/rusty-app/qml/Explorer.qml`, `crates/rusty-app/src/folders.rs`,
  `docs/planning/tickets/closed/TICKET-016-folders.md`.
- Promoted from intake: none; minted at TICKET-016's design on 2026-09-03 as the seal
  said ("file operations and git decorations become parts two and three, minted at
  design").
- Follow-ups opened: none.
