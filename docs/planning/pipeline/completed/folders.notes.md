---
title: Folders, part one: notes
pipeline_id: bb302c74-c180-4ce4-aea4-68a6ff889539
---

# Folders, part one: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices, the silent hook among them). Register:
  `AD-rusty-mcp-only-back-end-001` (the store), `AD-rusty-workspace-is-obsidian-001`,
  `PR-rusty-qml-component-scope-001`. Code read: `Explorer.qml` (rows built by `walk`,
  one delegate keyed on `kind`, `rowMenu`, the favorites section), `Main.qml` (`ui`
  state keys, `openPage`, `openView`, `openTerminal(program, name, session, cwd)`, the
  `Loader` per tab kind), `AgentTerminal.qml` (`startDir` from `cwd`), `desk.rs` (the
  bridge shape), `terminals.rs` (`load_state`/`save_state`), `RenderParams`.
- Sealed at 17:20 (relayed); the answers are on the spec's `sealed:` line.

## Phase 2: Design

- Architecture and data flow: a `Folders` QObject in the app (`src/folders.rs`) reads
  the disk: `list(path)` → JSON entries (name, path, kind, size; folders first, names
  without case, dotfiles skipped), `kindOf(path)` → `markdown`, `image`, `text` or
  `other` (extension first, then a sniff: a NUL or invalid UTF-8 in the first eight
  kilobytes means `other`), `readText(path)` up to a megabyte, `baseName`,
  `openExternally` (xdg-open, detached), `home`. The explorer keeps the roots the window
  hands it and a listing cache; `rebuild` appends the disk rows after the vault rows
  (`section`, then per root `root` and, expanded, `dir` and `disk` rows with depth). A
  click on a `disk` row raises `openFile`; the window opens a `file` tab (`FileTab.qml`:
  markdown rendered through `brain_render` with the text and the page style, a Source
  toggle; text as a `ListView` of numbered monospace lines; an image fitted) or, for
  `other`, hands the path to the desktop. The disk menu offers one entry per installed
  agent and a shell (each `openTerminal(program, "", "", dir)`), copy path (a hidden
  editor), reveal (the desktop's handler on the folder), Refresh, and on a root Remove.
  Roots persist under `roots` in the workspace state; expansion under `expanded`.
- File manifest:
  - `crates/rusty-app/src/folders.rs` (new), `src/main.rs` (`mod folders`),
    `build.rs` (the source and `qml/FileTab.qml`).
  - `crates/rusty-app/qml/Explorer.qml`: the roots, the disk rows, the disk menu, the
    Add folder button, the copy helper.
  - `crates/rusty-app/qml/FileTab.qml` (new).
  - `crates/rusty-app/qml/Main.qml`: `Folders`, `ui.roots`, `rootList`, `addRoot`,
    `removeRoot`, `openFile`, the `file` tab kind and its component, the `FolderDialog`,
    a palette command, the `root:` scene.
  - `crates/rusty-mcp/src/main.rs`: `RenderParams.markdown` (additive).
  - `scripts/screenshot.sh`: the `folders` scene (a root at the repository's `docs/`).
  - Phase 5: README, `docs/architecture.md`, ROADMAP, the wiki (`workspace-app.md`,
    `mcp-back-end.md`), two tickets minted (parts two and three).
- Store consequences: none; one state key (`roots`) in `~/.config/rusty/workspace.json`.
- Tool contract: `brain_render` gains an optional `markdown`; nothing renamed; 71 tools.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | the `folders` scene (a root below the vault, expanded); `ui.roots` by reading |
  | REQ-002 | `list_dir_sorts_folders_first_and_skips_dotfiles`, `kind_for_reads_the_extension_then_sniffs`, `read_text_stops_at_binary`; the scene with a markdown file open |
  | REQ-003 | reading: the menu's entries call `openTerminal` with the folder as `cwd`; `AgentTerminal.startDir` |
  | REQ-006 | review: disk rows never reach `openPage`, `brain_*` or the search |
- Risks: a huge folder lists synchronously (a directory with tens of thousands of
  entries would pause the UI for a moment; part two can page it); symlink loops are
  only walked by hand, one expansion at a time; the image tab loads asynchronously; no
  watcher, so Refresh is the way to see a change in part one.
- CodeGraph evidence: `openTerminal` has three callers (the top bar, the palette, the
  agent pane); the disk menu becomes the fourth. `RenderParams` is read by `brain_render`
  alone.

## Phase 3: Implement

- Built: the manifest as designed, plus the `folders` scene arguments (`root:<path>` and
  `file:<path>` scene parts, so `scripts/screenshot.sh <dir> "root:...,file:..."` shows a
  root and a file without a new script scene). `Folders` has three tests
  (`list_dir_sorts_folders_first_and_skips_dotfiles`, `kind_for_reads_the_extension_then_sniffs`,
  `read_text_stops_at_binary_and_marks_a_cut`); `render_text` on the brain manager has
  `render_text_renders_markdown_that_is_not_a_page`.
- Deviations: the section row ("Folders") is a row of the one list, as decided; the
  file tab's Source toggle shows the same numbered viewer the text kind uses.
- Defects on the way: `base_name("/")` returned an empty string (fixed, the test caught
  it); the file tab's `folders: folders` binding named its own required property, not
  the window's object (`PR-rusty-qml-component-scope-001` again; the instance is
  `diskFolders` now); the section row drew its name twice (the normal name text was
  not hidden for it; fixed).
- Evidence: `cargo test -p rusty-app folders` → 3 passed; `cargo test -p rusty-core
  render_text` → 1 passed; the scene `root:<repo>/docs,file:<repo>/docs/architecture.md`
  at 18:10 shows the Folders section, the expanded root with four folders and one file,
  and the file tab with the rendered markdown, the path, the kind, Source, Reload and
  Open outside.
- Fast gate: `bin/gate.sh --fast` → GATE GREEN [fast] at 18:14.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Boundary | Disk rows raise `openFile`, `openAgentAt`, `removeRoot`; none reaches `openPage`, a `brain_*` tool or the search; `currentFolder` derives from the current page slug alone, so "New note" on a disk row still lands in the vault root. | none | confirmed (REQ-006) |
| 2 | Scope | The file tab's `folders` binding named its own property, not the window's object. | high | fixed: the window's instance is `diskFolders` and both bindings point at it |
| 3 | UI | The section row showed the rename field because an empty `renaming` matched its empty path. | medium | fixed: renaming needs a non-empty name |
| 4 | Data safety | Part one writes nothing to the disk; `open_externally` spawns `xdg-open` detached with no inherited pipes. | none | confirmed |
| 5 | Performance | A listing is synchronous; a folder with tens of thousands of entries would pause the UI for a moment. | low | accepted for part one, noted on the ticket for part two |
| 6 | Rendering | The page renderer treats a single newline as a line break (Obsidian's default), so a repository README wraps where its source wraps. | low | accepted: the reading view of the vault does the same; the Source toggle shows the text |
| 7 | State | `expanded` now holds absolute paths beside vault paths; they cannot clash (vault paths never start with `/`). | none | confirmed |

- Post-implementation CodeGraph: `openTerminal` has four callers (the top bar, the
  palette, the agent pane, the explorer's disk menu); `Folders` is created in `Main.qml`
  alone; `render_text` has one caller, `brain_render`.

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-app folders` → 3 passed;
  `cargo test -p rusty-core render_text` → 1 passed; the workspace under the gate.
- Gate run: `bin/gate.sh --fast` GREEN; `bin/gate.sh --diff` below.
- Smoke evidence: the scene `root:/srv/stacks/rusty-v3/docs,file:/srv/stacks/rusty-v3/docs/architecture.md`
  at 18:12 on a fresh build: the Folders section under the vault tree, the `docs` root
  expanded with `architecture`, `design`, `planning`, `screenshots` and `architecture.md`
  (an `MD` badge), and the file tab with the name, the path, the kind, Source, Reload and
  Open outside above the rendered markdown; the scene log carries no QML warning.
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the scene: the Folders section, the `docs` root
  expanded; `roots` in the state by reading `Main.qml`). REQ-002 PASS (the three
  `folders` tests; the scene's rendered markdown tab; the text and image branches of
  `FileTab.qml` by reading; `other` goes to `openExternally` in `openFile`). REQ-003 PASS
  (the disk menu's `Instantiator` over `agents`, each `openTerminal(program, "", "",
  dir)`; `AgentTerminal.startDir` takes `cwd`). REQ-006 PASS (inspect finding 1).
  REQ-004 and REQ-005 are TICKET-019 and TICKET-020.
- Docs: README (a Folders section), `docs/architecture.md` (as built), ROADMAP, the wiki
  pages `workspace-app.md` and `mcp-back-end.md` through the OpenWiki run 18fe4f49
  (complete; the hook fed by hand as the bulletin describes).
- AAR: `AAR-016-folders.md`. Register: `AD-rusty-disk-is-not-the-store-001`,
  `PR-rusty-scratch-cleanup-without-a-glob-001`.
- Brain capture: the project page's timeline, after the commit.
- Archive: this pair to `completed/`. The completed spec of TICKET-015 carries its
  status line fixed in this commit (the check reads "Phase 5 — Complete PASS" exactly).
