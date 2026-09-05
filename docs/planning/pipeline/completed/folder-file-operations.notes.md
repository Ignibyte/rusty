---
title: Folders, part two — notes
pipeline: 1140d5ef-f95d-4c95-a05e-8e977e5af3f5
ticket: TICKET-019
---

# Folders, part two: notes

## Recall (2026-09-05)

- Bulletins: none critical; bulletin 2 (no synthetic input on Chad's desktop) shapes
  validation; `PR-rusty-probes-use-throwaway-rows-001` (probes create and delete their
  own data) shapes the tests: every test builds its own temporary tree and its own
  temporary trash root.
- Register: `AD-rusty-disk-is-not-the-store-001` — the disk is read by the app's own
  `Folders` type, never the back end, and part one writes nothing. Part two writes
  through the same type; the back end still never sees a root.
- 016's locked decisions 1, 2 and 5 still stand (the Rust type; disk rows in the one
  list; hidden entries skipped, listing cached until Refresh). Its named seams are this
  ticket's scope, word for word.
- Wiki: `workspace-app.md` describes the explorer's roots, `Folders`, and the read-only
  file tab, and says "part one writes nothing under it, and no root reaches a brain tool".
- Code read:
  - `folders.rs`: a cxx-qt bridge with five invokables (`list`, `kind_of`, `read_text`,
    `base_name`, `open_externally`) over pure functions (`list_dir`, `list_json`,
    `kind_for`, `read_text`, `base_name`, `home_dir`) and a `tree()` test helper that
    builds a temporary tree. The shape to extend.
  - `Explorer.qml`: one `ListView` of rows with kinds `page`, `folder`, `file` (vault),
    `section`, `root`, `dir`, `disk` (machine). Vault rows have `rowMenu` (new note, new
    folder, rename inline through `renaming`, move dialog, delete dialog, all through
    brain tools). Disk rows have `diskMenu` (agents, shell, open, open outside, copy path,
    reveal, refresh, remove root) — nothing that writes. Keys: Return, Left, Right; no F2,
    no Delete on any row. `refreshDisk()` drops the listing cache and rebuilds.
  - `FileTab.qml`: read-only; `Reload` calls `load()`; text as numbered lines, markdown
    rendered through `brain_render` with a Source toggle. No editor.
  - `NoteTab.qml`'s autosave: `dirty`, a 1500 ms `Timer` restarted on each keystroke,
    `save()` on it and on Ctrl+S, `reload()` only when not dirty, a "•" mark. Copied.
  - `rusty-app`'s dependencies already carry `chrono` (0.4) and `dirs` (6): the
    `.trashinfo` date is `chrono::Local::now()` formatted to the spec, and the trash root
    is `dirs::data_local_dir()`. (An earlier draft of this line said otherwise; the read
    landed after it was written.)

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/src/folders.rs` | six invokables — `create_file`, `create_dir`, `rename_entry`, `move_entry`, `trash`, `write_text` — each a thin wrapper returning JSON over a pure function; `validate_name`; `trash_to(path, trash_root)`; `write_atomic`; `trash_root()`; tests for each on the `tree()` helper |
| `crates/rusty-app/qml/Explorer.qml` | the disk menu gains New file, New folder, Rename…, Move to…, Delete; `renaming` and `rename()` branch on kind; `remove()` branches; a `diskMoveDialog`; F2 and Delete on the list; a `DragHandler` on disk rows with `indexAt` for the target; every write ends in `refreshDisk()` and surfaces `error` in `notice` |
| `crates/rusty-app/qml/FileTab.qml` | an `editing` mode for `text` and `markdown`: a `TextArea` in the terminal face, `dirty`, a 1500 ms autosave, Ctrl+S, an Edit/View button, Reload guarded by `dirty` |
| `crates/rusty-app/qml/Main.qml` | one scene, `file:edit`, that opens the current file tab's edit mode for the screenshot |

### The Rust functions

```
validate_name(name) -> Result<(), String>        empty, "." , "..", or containing '/' is refused
create_file(dir, name) -> Result<PathBuf>        refuses when the path exists
create_dir(dir, name) -> Result<PathBuf>         same
rename_entry(path, name) -> Result<PathBuf>      same parent, new name; refuses when the target exists
move_entry(path, into) -> Result<PathBuf>        into/<basename>; refuses when the target exists or `into` is inside `path`
trash_to(path, trash_root) -> Result<PathBuf>    files/<name>[ .N ] and info/<name>.trashinfo; rename, or copy then remove across devices
write_atomic(path, text) -> Result<()>           <dir>/.<name>.rusty-tmp then rename
```

Every invokable maps `Ok(p)` to `{"ok":true,"path":p}` and `Err(e)` to
`{"ok":false,"error":e}`; QML parses and either refreshes or shows the notice.

### The trash

`trash_root()` is `$XDG_DATA_HOME/Trash` or `~/.local/share/Trash`. `trash_to` makes
`files/` and `info/`, picks a name that does not exist in `files/` (`name`, then
`name.1`, `name.2`…), writes `info/<that>.trashinfo` with `[Trash Info]`,
`Path=<absolute, percent-encoded>`, `DeletionDate=<local civil time, ISO>`, then renames;
when `rename` fails with `EXDEV` it copies (a file, or a directory recursively) and
removes the source. The info record is written first so a crash between the two leaves a
record without a file, which file managers tolerate, rather than a file without a record.

### The drag

On a `disk` or `dir` row: `DragHandler { target: null; yAxis.enabled: true; xAxis.enabled: false }`
(rows move vertically). On release: `list.indexAt(0, mapped y)`, the row there must be
`dir` or `root` and share the source's root (the source path starts with that root's
path); then `folders.moveEntry(src, target.path)`.

### The file tab

`editing: false` → the existing viewers. `editing: true` → a `TextArea` bound to `text`,
`onTextChanged` marks `dirty` and restarts the timer unless `applying`; `save()` calls
`folders.writeText(path, editor.text)` and clears `dirty` on `ok`, else shows the error;
Ctrl+S saves; Reload is refused with a notice while dirty; the header shows "•" while
dirty. Markdown re-renders after a save so the Rendered view is current.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `folders::tests`: `create_file_and_dir`, `rename_and_move`, `trash_writes_info_and_moves`, `trash_picks_a_free_name` on a temp tree and a temp trash root |
| REQ-002 | `write_atomic_leaves_no_temp`; offscreen scene `file:edit` of the tab in edit mode |
| REQ-003 | reading: each QML write path calls `refreshDisk()` on `ok` |
| REQ-004 | reading of the handler and the root check; Chad's drag |
| REQ-005 | reading of the `Keys` handlers |
| REQ-006 | `refuses_existing_targets`; reading of the notice path |
| REQ-007 | `validate_name_refuses_separators_and_dots` |

### Store, tools, compatibility

None. No tool, no schema. The back end is not involved.

### Risks

- **Data safety.** Every write refuses an existing target, so nothing is overwritten;
  delete is a move to a trash a file manager can restore from; text writes are atomic.
  `move_entry` refuses a destination inside the source (a folder into itself).
- **The vault.** Disk writes are only offered on `disk`/`dir`/`root` rows; vault rows keep
  the brain tools. A root that *is* the vault folder is possible; the writes would bypass
  the index until the watcher's next burst — same as any external editor, and the
  watcher covers it. Noted in the wiki.
- **The trash across devices.** Copy-then-remove for a directory tree is recursive; a
  failure midway leaves a partial copy in the trash and the source intact — the safe
  side.
- **Keyboard.** F2 and Delete on the list; Ctrl+S in the tab; Escape leaves inline rename
  (already). The drag has a Move to… dialog beside it.
- **Theme.** No new colour; the editor takes the terminal face like the note editor.
- **No back end.** Only the markdown re-render needs it; the tab says so as it does now.

### CodeGraph

`codegraph_explore` over `Folders` before the change: the type has five invokables and no
callers in Rust beyond the bridge; QML callers are `Explorer.qml`, `FileTab.qml` and
`Main.qml`. A second pass after implementation goes in the ledger.

## Phase 3: Implement

Four files, as the manifest said. `folders.rs`: six invokables in the bridge, each one
call into a pure function and `outcome()` for the JSON; `validate_name`, `fresh`,
`create_file`, `create_dir`, `rename_entry`, `move_entry`, `trash_root`, `percent_encode`,
`trash_to` (with `copy_tree` and `remove_tree` for the `EXDEV` case), `write_atomic`; five
tests on temporary trees. `Explorer.qml`: `isDiskRow`, `diskResult` (every `ok` ends in
`refreshDisk()`), the four write helpers, `rootOf`, the disk branches in `rename` and
`remove`, F2 and Delete on the list, the `DragHandler` on disk rows, five menu items, the
trash wording in the delete dialog, `diskNameDialog` and `diskMoveDialog`. `FileTab.qml`:
the edit mode. `Main.qml`: the `file:<path>` and `file:edit` scenes. `qmllint` exit 0 on
all three.

Deviations: `move_entry` checks that the target is a folder before it checks for a folder
moving into itself (the test's first run found the other order; below, F1). A drop on the
row's own folder is a no-op rather than an "exists" notice (F2). `cargo fmt --all` ran.

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | `move_entry` tested "into itself" before "is a folder", so a file dropped on a file answered "a folder cannot move into itself" | **confirmed** by `rename_and_move_stay_inside_the_tree` on its first run; the folder check comes first |
| F2 | data safety | a drop on the row's own parent folder reached `fresh()` and came back as the error "exists" | **confirmed**; the drop returns before the move |
| F3 | correctness | the drag reads its release point from `centroid.position` inside `onActiveChanged` | rejected: `setActive(false)` fires before the point resets; the tab drag of TICKET-021 reads the same way |
| F4 | data safety | a grab cancelled mid-drag (the window losing focus) still drops where the pointer was | accepted: the drop is a move under the same root, put back by a drag or Move to…; noted here |
| F5 | correctness | inline rename on a disk row | rejected: `isRenaming` compares the absolute path, the field's `onAccepted` calls `explorer.rename`, which branches on `isDiskRow` before the vault's slug arithmetic |
| F6 | correctness | `Keys.onPressed` beside the list's `onReturnPressed`, `onRightPressed`, `onLeftPressed` | rejected: it accepts only F2 and Delete; the rename field, when focused, takes Delete first as any focused item does |
| F7 | keyboard first | `root` and `file` rows are outside F2 and Delete | accepted: a root leaves by its own menu, and a `file` row inside the vault has no tool, as in part one |
| F8 | correctness | the autosave timer firing after the tab leaves edit mode | rejected: `save()` returns when not dirty, and `toggleEditing` saves before it leaves |
| F9 | correctness | `file:edit` before the tab exists | rejected: `Qt.callLater`, and `load()` is synchronous, so the editor holds the text when it opens |
| F10 | theme | the editor binds `termFont` and `foreground`, the dirty mark `accent` | no new colour |
| F11 | data safety | a root that is the vault folder: a disk write bypasses the index | accepted as designed: the watcher's burst covers it as it covers any editor; in the wiki |
| F12 | data safety | `write_atomic` over a symlink renames a real file over the link | accepted: the link becomes a file, as most editors' atomic saves do; in the wiki |
| F13 | correctness | CodeGraph reports no covering tests for the `create_dir` and `move_entry` invokables | rejected: the bridge wrappers are one call each into `create_dir` and `move_entry`, which the tests cover |
| F14 | correctness | a long absolute path widens the delete dialog | accepted: no elision on a one-line label; the window is wide |
| F15 | prose | dialog strings, comments, the notice text | read against `no-ai-slop`; the notice for a refused Reload names what to do |
| F16 | correctness | part one already had a `file:` scene taking an absolute path; the new home-relative branch sat above it and shadowed it | **confirmed** (found at complete, before the wiki); one branch: an absolute path as given, anything else under `HOME` |

CodeGraph after the change: `Folders` carries eleven invokables; the six new ones have
callers only in `Explorer.qml` and `FileTab.qml`; the pure functions have no Rust callers
beyond the bridge and the tests; nothing outside `folders.rs` depends on them. The blast
radius matches the manifest.

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: first run `GATE RED [fast] at test`, one failure
  (`folders::tests::rename_and_move_stay_inside_the_tree`, F1); after the fix `GATE GREEN
  [fast]` — `rusty-app` 29 passed, `rusty-core` 248 passed, `rusty-mcp` 7 + 3 + 1 passed.
- `cargo build -p rusty-app -p rusty-mcp` (22:54:29), then `scripts/screenshot.sh <scratch>
  "file:.rusty/brain/concepts/compiled-truth.md" "file:.rusty/brain/concepts/compiled-truth.md,file:edit"`,
  offscreen against the scratch vault (the `file:` scene opens a path under the scratch
  `HOME`; the script seeds no plain file, so one of its own markdown pages served as the
  disk file). Both logs clean of `error|warning|TypeError|ReferenceError|Cannot assign|is
  not a type|Detected anchors`.
  - `…compiled-truth.md.png`: the tab as part one showed it, the header now with Edit.
  - `…compiled-truth.md-file-edit.png`: the header reads View, Source is hidden, the editor
    holds the file in the terminal face without a highlighter, no dirty mark — REQ-002's
    scene. "not connected" below it is the render's notice, as in part one's scenes.
- REQ-001, REQ-006 and REQ-007 rest on the five Rust tests; REQ-003 on `diskResult`
  (every `ok` calls `refreshDisk()`); REQ-004 on the handler's readings (F3, F4) and
  Chad's drag; REQ-005 on F5 to F7.
- `bin/gate.sh --diff` after the last gated edit: fmt, clippy, test, doc, shell-syntax,
  secrets, whitespace all ok, `receipt written: .git/rusty-gate-receipt`, `GATE GREEN [diff]`.
- F16 was a gated edit after that run; the gate ran again (below).
- `bin/gate.sh --diff` after F16: every step ok, `receipt written: .git/rusty-gate-receipt`,
  `GATE GREEN [diff]`.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-007 satisfied — REQ-001, REQ-006 and REQ-007 by the
  Rust tests, REQ-002 by the atomic-write test and the `file:edit` scene, REQ-003 to
  REQ-005 by the ledger's readings and Chad's use of the drag. None split, none waived.
- Wiki: run `37742133-5ad2-4a36-87ca-a9117a9a5a16`, `openwiki_finish` → `complete`; the
  folder-roots section gained part two, the invariant covers writes, a failure mode and the
  tests added; one claim re-anchored (its `FileTab` range had gone stale), three added, the
  scenes claim extended. The PostToolUse hook did not fire (sixth sighting; the receipt on
  disk was TICKET-023's); bulletin 3's recovery with the pair under `active/`, then
  `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-disk-writes-refuse-and-trash-001` in the AAR and the
  register. Brain: timeline entry on `projects/rusty-v3`.
