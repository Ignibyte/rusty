---
type: "Reference"
title: "Workspace app: Obsidian's layout with terminals inside"
openwiki_generated: true
sources:
  - id: openwiki-source-a2d65a21b1f78042c4d974ff
    resource: repo://crates/rusty-app/qml/AgentTerminal.qml
  - id: openwiki-source-1762a766e47a090b0ce4e932
    resource: repo://crates/rusty-app/qml/BookmarksPane.qml
  - id: openwiki-source-4345f12b3a27e0b9f51220b5
    resource: repo://crates/rusty-app/qml/Explorer.qml
  - id: openwiki-source-9b55c0d3f46c1b691eb75ceb
    resource: repo://crates/rusty-app/qml/FileTab.qml
  - id: openwiki-source-3b43dd803d4036a4a5fbc4bc
    resource: repo://crates/rusty-app/qml/GraphView.qml
  - id: openwiki-source-01a38728b296862b2b3bc449
    resource: repo://crates/rusty-app/qml/Main.qml
  - id: openwiki-source-d678395ec2ca71c73018a3fd
    resource: repo://crates/rusty-app/qml/NoteTab.qml
  - id: openwiki-source-05432e517df5ee62fccde076
    resource: repo://crates/rusty-app/qml/QuickSwitcher.qml
  - id: openwiki-source-6790183f51655ba192900138
    resource: repo://crates/rusty-app/qml/RightPane.qml
  - id: openwiki-source-5a3d0a4f21f2ef012ca2b079
    resource: repo://crates/rusty-app/qml/SearchPane.qml
  - id: openwiki-source-d5146c6223f7d1c9aa012aca
    resource: repo://crates/rusty-app/qml/SecretsPage.qml
  - id: openwiki-source-157820f2258f93d1ba08859f
    resource: repo://crates/rusty-app/qml/SettingsPage.qml
  - id: openwiki-source-f536fe8c8de4eb428d24ba4b
    resource: repo://crates/rusty-app/qml/TopBar.qml
  - id: openwiki-source-68599611588cfbbf1f2b222b
    resource: repo://crates/rusty-app/src/backend.rs
  - id: openwiki-source-c20a2a4e587e9ab45705b8d4
    resource: repo://crates/rusty-app/src/desk.rs
  - id: openwiki-source-6dd37e4946f07f310a54638b
    resource: repo://crates/rusty-app/src/folders.rs
  - id: openwiki-source-c8c0347aa7a687c601520d1a
    resource: repo://crates/rusty-app/src/main.rs
  - id: openwiki-source-c3978cc62c783d6d3ec4b39d
    resource: repo://crates/rusty-app/src/skin.rs
  - id: openwiki-source-720644bc52136dc05589b8d5
    resource: repo://crates/rusty-app/src/terminals.rs
  - id: openwiki-source-62f5347acdae1a6fb6fd8a74
    resource: repo://crates/rusty-app/src/theme.rs
generated: {by: "claude-code", at: "2026-09-03T23:11:49.896Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-03T23:11:49.896Z
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
- `src/skin.rs`: the look as data. `Roles` (a ground and three panel levels, two line
  weights, three text weights, an accent and its softer twin, gold, an alive colour,
  red, the six semantic colours, a face and a corner radius), the built-in `PRESETS`
  (Amber phosphor first), `from_omarchy` (the desktop palette and its ANSI colours
  mapped onto the roles), `parse_theme` and `from_file` for `~/.config/rusty/themes/`,
  `resolve` for a `Choice` (`preset`, `omarchy` or `file`, a name, the scanline switch)
  and `tokens`, which derives every colour the shell binds to under the older names
  and the new.
- `src/theme.rs` and `src/omarchy.rs`: `Theme` exposes the tokens, the faces, the radius
  and the switch as properties, `select()` switches the skin from the choice the shell
  keeps in the workspace state, `reload()` re-reads the desktop, and a watcher on
  `~/.config/omarchy/current` reloads on `omarchy theme set`; `omarchy.rs` reads the
  palette, the Alacritty font and colours, and writes the Konsole scheme. `main.rs` sets
  the application font from the saved skin before any item exists.
  Since TICKET-012 the theme also carries `baseSize` (12 to 18, default 14, or
  `RUSTY_TEXT_SIZE` when set, which then wins) and `scale` (`baseSize / 12`); every QML
  text size is `Math.round(n * theme.scale)` for the drawn value `n`, the reading view's
  render style takes its `size` from the same scale, and the terminal keeps the Alacritty
  font. `textSize` in the workspace state, Ctrl with plus, minus and zero, three palette
  commands and a stepper under "This machine" in Settings set it; a Rust test in
  `theme.rs` refuses a literal `pixelSize` anywhere in the QML.
- `src/desk.rs`: `Desk`, what the top bar reads: memory in use, the CPU's share, the
  clock, the login name. It asks the compositor for nothing; the Hyprland workspace strip
  went with TICKET-011, since waybar shows the workspaces.
- `src/terminals.rs`: `Terminals`, the tabs file (`~/.config/rusty/tabs.json`), the
  workspace state file (`~/.config/rusty/workspace.json`), tmux session names and
  listing, installed agents, desktop notifications.
- `src/markdown.rs`, `cpp/highlighter.*`, `cpp/tools.*`: the editor's highlighter and
  the window grab.
- `qml/Main.qml`: the layout and the tab model; `Explorer.qml`, `SearchPane.qml`,
  `NoteTab.qml`, `RightPane.qml`, `AgentTerminal.qml`, `QuickSwitcher.qml`,
  `CommandPalette.qml`, `Icon.qml`; the built-in views `TasksPage.qml`,
  `MemoryPage.qml`, `SkillsPage.qml`, `SecretsPage.qml`, `SettingsPage.qml`;
  `BookmarksPane.qml`, `GraphView.qml`, `TopBar.qml` and `Scanlines.qml`.

## Runtime flow

- Layout: a top bar (`TopBar.qml`: the brand, the command button, one glyph per agent
  CLI on `PATH` with a click for a new tab and a right-click for the agent pane, the
  vault's state, memory, CPU, the clock), a ribbon (new note, daily note, graph, Tasks,
  Memory, Skills, Secrets, Settings), a left sidebar (files, search, bookmarks), the main
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
- The graph (`GraphView`, tab kind `graph`, Ctrl+G, the ribbon, "Open local graph" on a
  page): `brain_graph` supplies nodes and edges; a force simulation on a timer
  (repulsion between every pair, springs on the edges toward the link distance, a pull
  to the centre, damping, cooling until the layout settles) lays them out; a `Canvas`
  draws links, dots sized by the square root of the degree, and labels that fade in
  with the zoom or show on hover; the panel folds into Filters, Groups (queries with a
  colour from the theme's palette, first match wins), Display and Forces, remembered in
  the workspace state under `graph`. A local graph is the same view with `around` set
  and follows the last page that was current.
- Search (`SearchPane`): the query goes to `brain_search` as typed, operators included;
  the `Aa` and `.*` chips send `case_sensitive` and `regex`; the bookmark chip keeps the
  search. Bookmarks (`BookmarksPane`): one JSON array under `bookmarks` in the workspace
  state, entries of kind `file`, `folder`, `search` or `heading` with a title; added, or
  removed when present, from the page menu, the explorer's row menu, the search pane,
  the outline's row menu and the palette; a click opens the page, reveals the folder,
  runs the search, or opens the page and scrolls to the heading once it has rendered.
  Settings renders the palette's command list as the Hotkeys table, the terminal keys
  added, with a filter field.
  The file and folder bookmarks are the favorites (TICKET-013): a star beside the note's
  reading toggle adds or removes the open page (Ctrl+D and a palette command do the
  same), the explorer gathers them in a Favorites section above the tree (click opens,
  right-click removes), and the quick switcher lists them first and starred on an empty
  query; nothing new is stored.
- The look (`TopBar`, the rail, the pane heads, the tree, the tab strip, the note's
  furniture, the assistant header, the toast, `Scanlines`) follows the design mock:
  uppercase micro-labels, square corners unless the skin sets a radius, the accent on
  the active thing, gold on titles and folders, the alive colour on links and state.
  Settings shows every skin `Theme.choices` lists; a pick goes through
  `win.selectTheme`, which stores `{source, name, scanlines}` under `theme` in the
  workspace state and calls `Theme.select`, so every token repaints at once; the face
  applies at the next launch. The note asks `brain_graph` around itself (depth 3) for
  the legend card's direct, related and distant counts.
- Keys are Obsidian's (Ctrl+O, Ctrl+P, Ctrl+N, Ctrl+E, Ctrl+W, Ctrl+Tab, Ctrl+Shift+F,
  Ctrl+,, Ctrl+G, Alt+Left/Right, F2) and are disabled while a terminal has focus; the
  four terminal keys (Ctrl+Shift+T, Ctrl+Shift+W, Ctrl+PgUp/PgDn) stay global.
- Screenshots: `RUSTY_SHOT=<png>` grabs the window through `Tools.grabWindow` after a
  delay and quits; `RUSTY_SHOT_SCENE` opens the switcher, the palette, the editor, a pane
  or a search first. `scripts/screenshot.sh` runs this against a scratch vault.

The Secrets page (`SecretsPage.qml`) masks every value. When the server reports no PIN it
offers to set one; with one set it asks for the PIN, sends it to `secret_unlock` and keeps
the token in the page's memory alone. Unlocked, each row gains a reveal (one row at a time,
through `secret_reveal`), a copy through a hidden text editor, and an inline edit saved on
Enter through `secret_update`; a Lock control and a Change PIN control sit in the lock
block. The page relocks on the expiry the unlock answered, when the window loses focus, on
Lock, and whenever a reveal or update fails, and it polls the status every few seconds
while a lockout runs. `pin_timeout_minutes` is among the known settings.

Folder roots (`Explorer.qml`, `FileTab.qml`, `src/folders.rs`) put folders from the
machine below the vault tree. "Add a folder" (the plus in the pane's header, or the
palette) opens a folder picker; the roots live under `roots` in the workspace state, per
machine, and a root's menu removes it. The disk is read by the app's `Folders` type: a
listing cached until Refresh (folders first, names without case, dotfiles skipped), a kind
by extension and then a sniff of the first eight kilobytes (`markdown`, `image`, `text`,
`other`), text up to a megabyte, and `xdg-open` for the rest. The disk rows join the
explorer's one list with their own kinds (`section`, `root`, `dir`, `disk`), so keys and
the current row work as they do for the vault. A click on a file opens a read-only `file`
tab: markdown rendered by `brain_render` given the text (a Source toggle shows the
numbered text), text as numbered monospace lines, an image fitted; any other kind goes to
the desktop. A folder's menu offers one entry per installed agent and a shell, each
`openTerminal` with that folder as the working directory, copy path, reveal in the file
manager, and Refresh. Links, backlinks, graph and search never see a root. File operations
and git decorations are TICKET-019 and TICKET-020.

## Invariants

- The app holds no store; every view calls tools and renders JSON.
- Every colour is a token derived from the skin's roles; no QML file names a colour of
  its own. Nothing is sent anywhere but the local back end.
- Inside an inline QML `Component`, shared objects are bound through the window
  (`theme: win.theme`), because an unqualified name finds the component's own property
  first.
- App state that is not the window geometry lives in the JSON files the Rust side
  owns, not in QtCore `Settings` (which rewrote string properties with their defaults).
- The app touches nothing under `~/.rusty` itself: the PIN, its hash and the secrets
  file are the back end's; a value reaches the page only through a tool answer.
- The disk is not the store: a folder root is read by the app alone, part one writes
  nothing under it, and no root reaches a brain tool.

## Failure modes

- No back end: pages show "waiting for rusty-mcp" and the tree keeps its last state.
- Qt's messages go to journald when stderr is not a tty: `journalctl --user -t rusty`
  or `QT_FORCE_STDERR_LOGGING=1`; `RUSTY_DEBUG=1` adds a line per event.
- Anchors inside a page do not scroll yet; live preview is not built.
- A bookmark keeps the path it was made with: a renamed or deleted page leaves it
  pointing at a page that is no longer there, and the user removes it.

## Extension points

- A new tab kind: a `Component` in `TabHost`, a title in `viewTitles`, a ribbon button.
- A new palette command: an entry in `commandList()`.
- A new role: `skin::Roles`, `skin::fill` for its default, `skin::tokens`, a `Theme`
  property, its use in QML. A new preset: an entry in `skin::PRESETS`.

## Tests

- `cargo test -p rusty-app`: the tokenizer, the tabs JSON, the skin (presets, the
  Omarchy mapping, theme files, tokens), the colour math, the desk readings.
- `scripts/screenshot.sh` for the visual record; pointer and keyboard walks are done
  by hand, never by synthetic input on the user's desktop.

## Primary sources

- `crates/rusty-app/qml/Main.qml`, `NoteTab.qml`, `Explorer.qml`, `RightPane.qml`, `AgentTerminal.qml`
- `crates/rusty-app/src/backend.rs`, `theme.rs`, `omarchy.rs`, `terminals.rs`
- `crates/rusty-app/cpp/highlighter.cpp`, `crates/rusty-app/cpp/tools.cpp`
