---
type: "Reference"
title: "Workspace app: Obsidian's layout with terminals inside"
openwiki_generated: true
sources:
  - id: openwiki-source-a2d65a21b1f78042c4d974ff
    resource: repo://crates/rusty-app/qml/AgentTerminal.qml
  - id: openwiki-source-01a38728b296862b2b3bc449
    resource: repo://crates/rusty-app/qml/Main.qml
  - id: openwiki-source-d678395ec2ca71c73018a3fd
    resource: repo://crates/rusty-app/qml/NoteTab.qml
  - id: openwiki-source-6790183f51655ba192900138
    resource: repo://crates/rusty-app/qml/RightPane.qml
  - id: openwiki-source-68599611588cfbbf1f2b222b
    resource: repo://crates/rusty-app/src/backend.rs
  - id: openwiki-source-040df95238fa90bd4e7ad29b
    resource: repo://crates/rusty-app/src/omarchy.rs
  - id: openwiki-source-720644bc52136dc05589b8d5
    resource: repo://crates/rusty-app/src/terminals.rs
generated: {by: "claude-code", at: "2026-09-03T05:07:54.592Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-03T05:10:15.800Z
---

# Workspace app: Obsidian's layout with terminals inside

## Purpose

The app (`crates/rusty-app`, binary `rusty`) is the place the vault is read and written.
It is laid out as Obsidian lays out a vault, so a reader of that app is at home, and it
adds what Obsidian lacks: agent terminals as tabs and as a pane beside the note, and
every view backed by the MCP server that agents share.

## Ownership

- `src/main.rs`: the Qt application, the QML engine, `COLORSCHEMES_DIR` for the
  terminal widget.
- `src/backend.rs`: `Backend`, the MCP client: one Streamable HTTP session on a tokio
  runtime, reconnecting every three seconds; `call(tool, argsJson)` returns an id and
  the reply arrives through `result(id, tool, json, ok)`; `resources/list_changed`
  becomes `dataChanged`.
- `src/theme.rs` and `src/omarchy.rs`: `Theme`, the Omarchy colours and the tokens read
  from the theme's `obsidian.css` and Alacritty palette (surface, line, muted, faint,
  code, headings, ANSI colours), the terminal font and a generated Konsole scheme; a
  watcher on `~/.config/omarchy/current` reloads on `omarchy theme set`.
- `src/terminals.rs`: `Terminals`, the tabs file (`~/.config/rusty/tabs.json`), the
  workspace state file (`~/.config/rusty/workspace.json`), tmux session names and
  listing, installed agents, desktop notifications.
- `src/markdown.rs`, `cpp/highlighter.*`, `cpp/tools.*`: the editor's highlighter and
  the window grab.
- `qml/Main.qml`: the layout and the tab model; `Explorer.qml`, `SearchPane.qml`,
  `NoteTab.qml`, `RightPane.qml`, `AgentTerminal.qml`, `QuickSwitcher.qml`,
  `CommandPalette.qml`, `Icon.qml`; the built-in views `TasksPage.qml`,
  `MemoryPage.qml`, `SkillsPage.qml`, `SecretsPage.qml`, `SettingsPage.qml`.

## Runtime flow

- Layout: a ribbon (new note, daily note, palette, Tasks, Memory, Skills, Secrets, one
  button per agent CLI on `PATH`, Settings), a left sidebar (files, search), the main
  area (tab strip and a stack of `TabHost`s), a right sidebar (backlinks with context,
  outgoing links, outline, tags as a tree with counts, an agent pane), a status bar
  with the counts.
- Tabs: one `ListModel` of `{kind, title, slug, session, program, cwd, pinned}`; kinds
  are `page`, `terminal` and the built-in views. Tabs persist across runs; a file from
  before the workspace loads as terminals. Opening a page from the explorer, search, a
  link or the switcher navigates the current page tab unless it is pinned.
- A page tab (`NoteTab`) asks `brain_render` with the theme's style and
  `brain_get_links`, shows the inline title (Enter renames through `brain_rename`), the
  properties, then the reading view as one rich-text block per top-level section, or
  the source in a `TextArea` with the highlighter. The properties block is the editor:
  each value edits by its type (a text or date field, a number, a checkbox, list chips
  with add and remove; tag chips also open a `tag:` search), a row can be removed, and
  "Add property" adds a key of a chosen type, all through `brain_set_property` and
  `brain_remove_property`; the page re-renders on the change notification. A `#tag` in
  the reading view, a chip, or a row of the Tags pane puts `tag:<name>` into the search
  pane. Edits autosave after 1.5 s and on
  Ctrl+S through `brain_write_page`; a `dataChanged` reloads only when the editor is
  clean. Each tab keeps its own history.
- Terminals (`AgentTerminal`): `qmltermwidget` running `tmux new-session -A -s
  <session> -c <dir> <program>` with `set-titles on`, so Claude Code's and Codex's
  titles reach the tab; output in a hidden tab marks it unread, a title that asks for
  attention raises a desktop notification once per title and at most once a minute. A
  terminal starts its session when first shown.
- Keys are Obsidian's (Ctrl+O, Ctrl+P, Ctrl+N, Ctrl+E, Ctrl+W, Ctrl+Tab, Ctrl+Shift+F,
  Ctrl+,, Alt+Left/Right, F2) and are disabled while a terminal has focus; the four
  terminal keys (Ctrl+Shift+T, Ctrl+Shift+W, Ctrl+PgUp/PgDn) stay global.
- Screenshots: `RUSTY_SHOT=<png>` grabs the window through `Tools.grabWindow` after a
  delay and quits; `RUSTY_SHOT_SCENE` opens the switcher, the palette, the editor, a pane
  or a search first. `scripts/screenshot.sh` runs this against a scratch vault.

## Invariants

- The app holds no store; every view calls tools and renders JSON.
- Every colour is a theme token; nothing is sent anywhere but the local back end.
- Inside an inline QML `Component`, shared objects are bound through the window
  (`theme: win.theme`), because an unqualified name finds the component's own property
  first.
- App state that is not the window geometry lives in the JSON files the Rust side
  owns, not in QtCore `Settings` (which rewrote string properties with their defaults).

## Failure modes

- No back end: pages show "waiting for rusty-mcp" and the tree keeps its last state.
- Qt's messages go to journald when stderr is not a tty: `journalctl --user -t rusty`
  or `QT_FORCE_STDERR_LOGGING=1`; `RUSTY_DEBUG=1` adds a line per event.
- Anchors inside a page do not scroll yet; live preview is not built.

## Extension points

- A new tab kind: a `Component` in `TabHost`, a title in `viewTitles`, a ribbon button.
- A new palette command: an entry in `commandList()`.
- A new theme token: `omarchy::tokens`, a `Theme` property, its use in QML.

## Tests

- `cargo test -p rusty-app`: the tokenizer, the tabs JSON, the tokens.
- `scripts/screenshot.sh` for the visual record; pointer and keyboard walks are done
  by hand, never by synthetic input on the user's desktop.

## Primary sources

- `crates/rusty-app/qml/Main.qml`, `NoteTab.qml`, `Explorer.qml`, `RightPane.qml`, `AgentTerminal.qml`
- `crates/rusty-app/src/backend.rs`, `theme.rs`, `omarchy.rs`, `terminals.rs`
- `crates/rusty-app/cpp/highlighter.cpp`, `crates/rusty-app/cpp/tools.cpp`
