---
title: Folders, part two: file operations
pipeline_id: 1140d5ef-f95d-4c95-a05e-8e977e5af3f5
status: Phase 5 — Complete PASS
ticket: TICKET-019
ticket_doc: docs/planning/tickets/open/TICKET-019-folder-file-operations.md
aar: docs/planning/knowledge/aar/AAR-019-folder-file-operations.md
sealed: minted at TICKET-016's design on 2026-09-03 under its seal ("file operations and git decorations become parts two and three"); no new tab, store or dependency here
created: 2026-09-05
---

# Folders, part two: file operations: spec

## Intent

Part one (TICKET-016) reads the disk. The explorer is half a file explorer until it can
change what it shows, and an agent's repository needs a rename or a delete now and then
without leaving the app. This part makes the disk rows act: new file, new folder, rename,
move, delete to the trash, and a text file editable in its tab.

## Scope

- In: six writes on `Folders` (create file, create folder, rename, move, trash, write
  text) as pure Rust with tests on a temporary tree; the disk menu's new entries; F2 and
  Delete on rows; move by drag between disk rows and a Move to… dialog for the keyboard;
  the file tab's edit mode with autosave, Ctrl+S, a dirty mark and Reload; every write
  refreshing the listing.
- Out (named seams, not forgotten): git (TICKET-020); permissions and ownership; the
  vault's own files (the brain tools stay their path); a watcher on the roots (Refresh
  stands in, as part one decided); editing an image or a binary; undo beyond the trash.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN new file, new folder, rename, move or delete is used on a disk row, it shall act on the disk under that root, and delete shall move the entry to the XDG trash with a `.trashinfo` record. | Rust tests on a temporary tree with the trash pointed at a temporary root |
| REQ-002 | WHEN a text or markdown file's tab is put in edit mode and edited, it shall save to disk 1.5 s after the last keystroke and on Ctrl+S, atomically, and Reload shall re-read the file when the tab is not dirty. | Rust test of the atomic write; offscreen scene of the edit mode |
| REQ-003 | WHEN any write completes, the explorer shall drop its cached listing for the root and rebuild, so the tree reflects the disk. | reading: every write path ends in `refreshDisk()` |
| REQ-004 | WHEN a disk row is dragged onto a folder or root row under the same root and released, the entry shall move into that folder. | reading; smoke by Chad |
| REQ-005 | WHEN F2 or Delete is pressed on the current row, it shall rename inline or open the delete dialog, for disk rows and vault rows alike. | reading |
| REQ-006 | WHEN a write fails (a name that exists, a permission), the explorer shall show the error in its notice and change nothing else. | Rust tests of the error paths; reading of the notice path |
| REQ-007 | WHEN a name given to create or rename contains a path separator or is empty, it shall be refused before touching the disk. | Rust test |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Every disk write is a `Folders` invokable over a pure Rust function returning `{ok, path}` or `{ok: false, error}` as JSON | `AD-rusty-disk-is-not-the-store-001`: the disk is the app's, not the back end's; a pure function is what a temp-tree test can hold | New MCP tools (the store's path, and a round trip per keystroke) |
| 2 | Delete is the XDG home trash, written in Rust: `files/` and `info/<name>.trashinfo` under `$XDG_DATA_HOME/Trash` or `~/.local/share/Trash`, a unique name on collision, copy-then-remove across devices | Restorable from any file manager; no new crate, so no seal; a `trash_root` parameter makes it testable | `gio trash` (a spawn, untestable without touching the real trash); the `trash` crate (a dependency) |
| 3 | Text writes are atomic: a sibling temp file, then `rename` | A crash mid-write must not leave a truncated file the agent then reads | `fs::write` in place |
| 4 | Move by drag copies the tab strip's `DragHandler` shape; the drop target is the row under the pointer via `indexAt`, accepted only when it is a `dir` or `root` under the same root | The pattern is proven twice now; same-root keeps a drag from crossing into the vault or another root | `Drag`/`DropArea` |
| 5 | F2 and Delete act on the current row for disk and vault rows alike | Keyboard first (§10); giving only disk rows keys would make the two halves of one list behave differently | Disk rows only |
| 6 | The file tab's edit mode is a plain `TextArea` in the terminal face, no highlighter | The note editor's highlighter is Obsidian-flavoured markdown; a `.rs` or a `.toml` wants none of it, and a highlighter per language is a product of its own | Reusing the note editor |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-019-folder-file-operations.md`
- Part one: `docs/planning/pipeline/completed/folders.spec.md` (its locked decisions 1, 2, 5 still stand)
- Architecture: `openwiki/workspace-app.md` (the explorer and the file tab), `AD-rusty-disk-is-not-the-store-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; covered by 016's seal |
| 2 Design | Manifest, the Rust functions, the drag, the tab, regression table | design actionable |
| 3 Implement | `folders.rs`, `Explorer.qml`, `FileTab.qml`, `Main.qml` (a scene) | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over `folders.rs` | confirmed findings resolved |
| 4 Validate | The tests, an offscreen scene, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
