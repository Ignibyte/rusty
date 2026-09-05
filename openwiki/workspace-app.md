---
type: "Reference"
title: "Workspace app: Obsidian's layout with terminals inside"
openwiki_generated: true
sources:
  - id: openwiki-source-4059556410fe6db8498fe8e9
    resource: repo://crates/rusty-app/build.rs
  - id: openwiki-source-a2d65a21b1f78042c4d974ff
    resource: repo://crates/rusty-app/qml/AgentTerminal.qml
  - id: openwiki-source-1762a766e47a090b0ce4e932
    resource: repo://crates/rusty-app/qml/BookmarksPane.qml
  - id: openwiki-source-5db647af1e42157766e439f7
    resource: repo://crates/rusty-app/qml/DecisionsPage.qml
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
  - id: openwiki-source-b219b7cb57258d9cb096d197
    resource: repo://crates/rusty-app/qml/SkillsPage.qml
  - id: openwiki-source-deeb404896a39245dc19d37e
    resource: repo://crates/rusty-app/qml/Splitter.qml
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
  - id: openwiki-source-c7501cab00d475ec77094adb
    resource: repo://crates/rusty-core/src/brain/mod.rs
  - id: openwiki-source-d4dc2c7ea0d931bfc9466b41
    resource: repo://scripts/screenshot.sh
generated: {by: "claude-code", at: "2026-09-05T04:22:26.145Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-05T04:22:26.145Z
---

# Workspace app: Obsidian's layout with terminals inside

## Purpose

The app (`crates/rusty-app`, binary `rusty`) is the place the vault is read and written.
It is laid out as Obsidian lays out a vault, so a reader of that app is at home, and it
adds what Obsidian lacks: agent terminals as tabs and as a pane beside the note, and
every view backed by the MCP server that agents share.

## Ownership

- `src/main.rs`: the Qt application, the QML engine, `COLORSCHEMES_DIR` for the
  terminal widget, and the `rusty <name>` command path that runs before any of it.
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

- The binary is two programs. Before Qt starts, `rusty <name> [args]` looks in the active
  skills store for a script of that basename; on a hit it execs `rusty-cli scripts run`
  and never creates a window, so `rusty usb-reset` is a command and `rusty` alone is the
  app (TICKET-010). A name that matches nothing falls through to the window, which is why
  an unknown command opens the workspace rather than reporting an error.
- Every drag handle is `Splitter.qml` (TICKET-023), listed in `build.rs` and shared by the
  sidebars and the Skills page. The owner binds `value`, `min`, `max` and `invert` and
  applies the result in `onMoved`; the handle clamps and reports but never writes a value
  itself, which is what lets one component serve a window pane and a page. It measures
  in scene coordinates — `mapToItem(null, …)` at press and at every move — not in its
  own frame (TICKET-022): the pane resize moves the handle under the pointer, so a delta
  measured from it is measured from a moving origin and the drag only tracked while the
  pointer stayed inside the 7px strip. The sidebars bind 180–600 px on the left and
  200–700 px on the right, the right one inverted so a drag left grows it; both widths
  persist in the workspace state.
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
  link or the switcher navigates the current page tab unless it is pinned. Tabs reorder
  by drag (TICKET-022): a `DragHandler` on each tab — left button, x axis only, no
  target of its own — records the origin when it activates and, on release, maps its
  centroid into the row and finds the tab whose span holds it (`dropIndexAt`, which
  skips the row's spacer and `+` because only delegates carry an `index`), then calls
  the same `moveTab` the Ctrl+Shift+PgUp/PgDown keys and the tab menu use, so the order
  persists as it already did. The tap handlers are untouched, and a press that becomes
  a drag never fires a select. The strip's `+` opens a menu under itself: the page
  switcher first (Ctrl+T, unchanged), one item per agent the top bar lists (an
  `Instantiator` inserted between two separators — a `Repeater` does not parent items
  into a `Menu`), then the custom terminal dialog (Ctrl+Shift+T, unchanged); its tooltip
  names both keys. The rename dialog lays its field out in a `ColumnLayout` the way the
  new-terminal dialog does, because a bare child with an explicit `width` does not size
  a `Dialog`'s content and the field used to spill past the edge.
- A page tab (`NoteTab`) asks `brain_render` with the theme's style and
  `brain_get_links`, shows the inline title (Enter renames through `brain_rename`), the
  properties, then the reading view as one rich-text block per top-level section, or
  the source in a `TextArea` with the highlighter. The properties block is the editor:
  each value edits by its type (a text or date field, a number, a checkbox, list chips
  with add and remove; tag chips also open a `tag:` search), a row can be removed, and
  "Add property" adds a key of a chosen type, all through `brain_set_property` and
  `brain_remove_property`; the page re-renders on the change notification. A `#tag` in
  the reading view, a chip, or a row of the Tags pane puts `tag:<name>` into the search
  pane. Tagging is deliberate as well as inline (TICKET-024): Tags is one of the types
  "Add property" offers, its key fixed to `tags` and its value a list (a page that has
  `tags` gets that row's field focused instead); the tags row's field completes from the
  tag index the window holds — a substring match without case, the page's own tags left
  out, by count then name, eight at most, nothing picked until Down, Enter adding the
  text as typed when nothing is picked, Tab taking the pick or the first; the Tags pane
  tags the open page from a `+` on hover, a right-click menu or `T` on the selected
  row; "Tags: Tag this page" puts the cursor in the field, adding the property first
  when the page has none. One function writes, `tagThePage` (a leading `#` dropped, a
  duplicate refused without case, a scalar `tags:` kept as one tag), and the counts,
  `tag:` search and the graph follow the same change notification. Edits autosave
  after 1.5 s and on Ctrl+S through `brain_write_page`; a `dataChanged` reloads only
  when the editor is clean. Each tab keeps its own history.
- Terminals (`AgentTerminal`): `qmltermwidget` running `tmux new-session -A -s
  <session> -c <dir> <program>` with `set-titles on`, so Claude Code's and Codex's
  titles reach the tab; output in a hidden tab marks it unread, a title that asks for
  attention raises a desktop notification once per title and at most once a minute. A
  terminal starts its session when first shown. Its clipboard (TICKET-021) is bound on
  the widget itself, because the workspace's shortcuts stand down while a terminal has
  focus and a window-level `Shortcut` would never fire: Ctrl+Shift+C and Ctrl+Shift+V
  call the exported `copyClipboard` and `pasteClipboard`, attached with `Keys.priority:
  BeforeItem` so plain Ctrl+C still reaches the shell as an interrupt. A `TapHandler`
  accepting only the middle and right buttons pastes the primary selection
  (`pasteSelection`) or opens a Copy/Paste menu; it takes an exclusive grab
  (`ReleaseWithinBounds`) because Konsole's `TerminalDisplay` pastes on a middle click
  natively and a passive area above it would paste twice, and a left-button drag still
  selects because that button is never accepted. Copy in the menu follows the widget's
  `copyAvailable` signal, read through `Connections { ignoreUnknownSignals: true }` as
  every other third-party signal here is.
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
the current row work as they do for the vault. A click on a file opens a `file` tab:
markdown rendered by `brain_render` given the text (a Source toggle shows the numbered
text), text as numbered monospace lines, an image fitted; any other kind goes to the
desktop. A folder's menu offers one entry per installed agent and a shell, each
`openTerminal` with that folder as the working directory, copy path, reveal in the file
manager, and Refresh. Links, backlinks, graph and search never see a root.

Part two (TICKET-019) writes the disk through the same `Folders` type, six invokables
that answer `{ok, path}` or `{ok, error}`: `createFile`, `createDir`, `renameEntry`,
`moveEntry`, `trash`, `writeText`. A folder's menu adds New file…, New folder…, Rename…,
Move to… and Delete; F2 renames inline and Delete opens the delete dialog on the current
row, disk or vault; a disk row drags onto a folder or root row under the same root
(`DragHandler`, no target of its own, the drop found by `indexAt` — never across roots,
never onto the vault, a drop on its own folder a no-op). Every write refuses an existing
target; a name that is empty, `.`, `..` or holds a slash is refused before the disk is
touched; Delete moves the entry to the XDG home trash (`$XDG_DATA_HOME/Trash`, else
`~/.local/share/Trash`) with its `.trashinfo` record written first and a copy across
devices, so a file manager can restore it. On `ok` the explorer drops the root's listing
cache and rebuilds; an error shows in its notice and changes nothing else. A text or
markdown file's tab has Edit: a `TextArea` in the terminal face with no highlighter, a
dirty mark, a save one and a half seconds after the last keystroke and on Ctrl+S through
`writeText` (a sibling temporary file renamed over the path, the mode kept), Reload
refused with a notice while dirty, the rendered view refreshed after a save.

Part three (TICKET-020) marks a root that lies in a git repository. `Folders.gitStatus`
runs `git --no-optional-locks rev-parse --show-toplevel` and then `status
--porcelain=v2 --branch -z --untracked-files=all -- .` in the root, parses the porcelain
in Rust and answers `{repo, branch, files, dirs}`: paths relative to the root (a root that
is a subfolder of a repository sees its own subtree), `M` for any change to a tracked
file, `A` for an added one, `?` for an untracked one, and every folder above a change
folded to the strongest state below it. The explorer fetches it once per root in
`rebuild`, keeps it beside the listing cache and drops both on Refresh, a root change and
every disk write; the rows only read it. A file shows its letter beside the extension and
takes the colour (`gold`, `alive`, `accentSoft`), a folder holding a change shows a dot,
the root row shows the branch in `faint` (`detached` when HEAD is). Outside a repository,
or without `git`, the answer is `{repo: false}` and the tree is part two's. Nothing is
written to the repository: `--no-optional-locks` keeps `status` from refreshing the index
on disk.

The Skills tab (`SkillsPage.qml`) carries a Scripts section beside the skill list, fed by
`script_list` and `script_view` and saved through `script_update`. A script is edited
where its skill is, because that is where it lives on disk, and Run opens it in a terminal
tab rather than capturing its output in the page — the same script the command line runs,
shown running. The list pane resizes through a `Splitter` clamped 200–600 px, and the
Skills and Scripts sections collapse from their headers — a chevron and a label, focusable,
toggled by click or by Enter and Space — with the open section taking the freed height
(TICKET-023). The split and both open states travel as one JSON string, `savedLayout` in
and `layoutChanged` out with a guard against echoing while applying, and live as
`ui.skillsLayout` in the workspace state the way `graph` and `bookmarks` do.

The Decisions view (`DecisionsPage.qml`, opened from the ribbon, the palette or
`openView("decisions")`) is fed by one tool, `brain_due`: the follow-ups due first, then
every decision with its status and dates; a click opens the page. The graph view draws a
decision's typed edges (a `kind` other than `link`: consulted, supersedes, follows up)
dashed in the accent colour behind a "Decision edges" filter that persists with the other
graph settings. The loop itself is the back end's (`mcp-back-end.md`); the hooks that make
an agent run it are in `workflow-and-gates.md`.

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
- The disk is not the store: a folder root is read and written by the app alone, no root
  reaches a brain tool, and no disk write overwrites anything — an existing target is
  refused, a delete is a move to the trash, a text save is an atomic rename. A root's git
  status is read the same way and the repository is never written.
- The command path never starts Qt. It resolves and execs before any Qt object exists,
  so a script inherits the terminal it was typed in rather than the app's environment.

## Failure modes

- No back end: pages show "waiting for rusty-mcp" and the tree keeps its last state.
- Qt's messages go to journald when stderr is not a tty: `journalctl --user -t rusty`
  or `QT_FORCE_STDERR_LOGGING=1`; `RUSTY_DEBUG=1` adds a line per event.
- Anchors inside a page do not scroll yet; live preview is not built.
- A bookmark keeps the path it was made with: a renamed or deleted page leaves it
  pointing at a page that is no longer there, and the user removes it.
- `rusty <name>` with a name no script answers to opens the workspace instead of
  reporting an unknown command, because the check is a lookup and its miss is the app's
  ordinary start. A script that exists but is pending exits 126 with the reason.
- A disk write that fails (a name in use, a permission, a root that vanished) comes back
  as `{ok: false, error}` and the explorer shows the reason in its notice; a root that is
  the vault folder itself takes disk writes past the index until the watcher's next
  burst, as any outside editor does; a text save over a symlink leaves a real file where
  the link was.
- A `git` that is missing, a root outside any repository, or a root whose real path git
  does not report as under its top level (a bind mount) all answer `{repo: false}`: the
  tree shows no mark rather than a wrong one, and nothing is reported.

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
- `cargo test -p rusty-app folders::` covers the listing, the kind sniff, the text cut,
  and the writes on temporary trees: names refused, existing targets refused, rename and
  move inside the tree, the trash record and its free-name suffix, the atomic write
  keeping the mode. `scripts/screenshot.sh <out> "file:<path>,file:edit"` photographs the
  edit mode.
- `cargo test -p rusty-app folders::` also covers the git read: the porcelain v2 parser on
  a fixture (branch, `1`, `2`, `u` and `?` records, a detached head), the folding of
  folders to the strongest state, a temporary repository built with `git init` under a
  temporary tree with `HOME` and `GIT_CONFIG_GLOBAL` pointed away from the machine's
  (modified, added, untracked, a subfolder root), and a plain folder.
  `scripts/screenshot.sh <out> "root:repo,expand:repo/src"` photographs the marks against
  the repository the script seeds.
- `cargo test -p rusty-core brain::tests::tags_index_search_and_properties` covers the
  deliberate tag path's back half: a `tags` list set through the property shows in the
  counts and in `tag:` search, and a tag removed through the property leaves both.
  `scripts/screenshot.sh <out> "tagfield:r" "right:tags"` photographs the completion
  list and the pane.

## Primary sources

- `crates/rusty-app/qml/Main.qml`, `NoteTab.qml`, `Explorer.qml`, `RightPane.qml`, `AgentTerminal.qml`
- `crates/rusty-app/src/backend.rs`, `theme.rs`, `omarchy.rs`, `terminals.rs`
- `crates/rusty-app/cpp/highlighter.cpp`, `crates/rusty-app/cpp/tools.cpp`
