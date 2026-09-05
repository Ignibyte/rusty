---
title: Workspace chrome fixes — notes
pipeline: e8f444c2-71de-4e14-8e1a-fff6b5c7ca14
ticket: TICKET-022
---

# Workspace chrome fixes: notes

## Recall (2026-09-05)

- Bulletins: none critical. Bulletin 2 again shapes validation: no synthetic input on
  Chad's desktop; the dialog and the menu are photographed offscreen, the drags are
  verified by reading and by Chad.
- Register: nothing on drag, splitters or tabs. `PR-rusty-probes-use-throwaway-rows-001`
  applies to any probe (scratch vault only). `AD-rusty-workspace-is-obsidian-001`: tabs
  are the workspace's unit, so reordering them by drag is the Obsidian behaviour the app
  claims.
- Wiki: `workspace-app.md` describes the tab `ListModel` and that tabs persist; the
  splitters and the strip's `+` are not described. The Settings page claims tab drag.
- Nearest completed notes: `knowledge-workspace-shell` (built the strip and the
  splitters), `top-bar-command-and-agents` (how the agents are enumerated:
  `win.agents`, `agentGlyphs`, `agentNames`, `agentLabel()`).
- Code read, with what each read settled:
  - `Main.qml` `Splitter` (`component Splitter: Item`, a 7px strip with a `MouseArea`):
    `startX = mouse.x` at press, `delta = mouse.x - startX` at move. Both are in the
    MouseArea's own frame; the handle moves with `ui.leftWidth`, so the frame moves under
    the pointer and the delta is measured from a moving origin. **Cause of bug 1 found.**
  - The tab delegate: `HoverHandler` plus three `TapHandler`s (left select, middle close,
    right menu). No `DragHandler` anywhere in `Main.qml`. `moveTab(from, to)` exists and
    is bound to Ctrl+Shift+PgUp/PgDown and the tab menu. `SettingsPage.qml:88` says "drag
    a tab or a task's handle to reorder". **Bug 2 confirmed: the feature is documented and
    absent.**
  - `TasksPage.qml`'s grip: `DragHandler { target: null; xAxis.enabled: false;
    onActiveChanged: … mapToItem(list.contentItem, …) … page.move(from, dropIndex) }`.
    The horizontal twin of this is the whole of REQ-002.
  - `renameDialog`: `Dialog { … TextField { id: renameField; width: 320 } }` — a bare
    child with an explicit width and no layout; `newTabDialog` beside it uses a
    `ColumnLayout` with `Layout.preferredWidth: 320` and lays out correctly. **Cause of
    bug 3 found.**
  - The strip's `+`: `SideTab { icon: "plus"; tip: "New tab (Ctrl+T)"; onClicked:
    switcher.show() }`. It opens the page switcher and nothing else. The top bar's agent
    glyphs call `win.openTerminal(p, "", "", "")`; the palette builds one command per
    agent the same way. **Bug 4 is a missing menu, not a missing capability.**

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/qml/Main.qml` | `Splitter`: scene-frame coordinates. Tab delegate: a `DragHandler` and a drop-index walk over `tabRow`. `renameDialog`: a `ColumnLayout`. The strip's `+`: a `Menu` (`plusMenu`) listing the switcher, the agents, and the custom terminal. |
| `crates/rusty-app/qml/SettingsPage.qml` | The reordering sentence, made true. |

### The four mechanisms

1. **Splitter.** At press, `startX = sp.mapToItem(null, mouse.x, 0).x`; at move,
   `x = sp.mapToItem(null, mouse.x, 0).x`, `delta = x - startX`. The scene frame does
   not move when the handle does. Clamps unchanged (180–600 left, 200–700 right).
2. **Tab drag.** In the delegate, a `DragHandler { target: null; yAxis.enabled: false }`.
   On `active` rising: remember `from = tabItem.index`. On falling: map the centroid to
   `tabRow`, walk `tabRow.children`, find the delegate whose x-range holds the point,
   and `win.moveTab(from, to)` when they differ. The three `TapHandler`s stay; a drag
   past the threshold cancels the tap, so a drag does not select (REQ-006).
3. **Rename.** `Dialog { ColumnLayout { TextField { Layout.preferredWidth: 320 } } }`,
   the shape of `newTabDialog`.
4. **The plus.** `SideTab { onClicked: plusMenu.popup() }` with
   `Menu { id: plusMenu; MenuItem "Note or page… (Ctrl+T)" → switcher.show();
   MenuSeparator; Repeater over win.agents → MenuItem agentLabel(a) → openTerminal(a);
   MenuSeparator; MenuItem "Custom terminal… (Ctrl+Shift+T)" → newTabDialog.openFresh() }`.
   A `Repeater` inside a `Menu` is the Qt Quick Controls way to build items from a
   model; `Instantiator` is the alternative if the Repeater does not parent items into
   the menu, and the inspect phase will check which the installed Qt does.

### Store, tools, compatibility

None. `moveTab` already persists the order; nothing new is stored.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | reading: both coordinates come from `mapToItem(null, …)`; Chad's drag |
| REQ-002 | reading: the handler and the walk; Chad's drag |
| REQ-003 | `scripts/screenshot.sh` scene that opens the rename dialog (a new `RUSTY_SHOT_SCENE` value, `rename`) |
| REQ-004 | reading; a screenshot scene that opens the plus menu (`plus`) |
| REQ-005 | reading of the sentence |
| REQ-006 | reading: the tap handlers are untouched, the drag handler moves nothing itself |

### Risks

- A `DragHandler` on the delegate and the left-button `TapHandler` on the same item:
  Qt's pointer handlers cooperate — a tap that exceeds the drag threshold becomes a drag
  and the tap is cancelled. The Tasks page relies on the same and works.
- Walking `tabRow.children` includes the trailing spacer `Item` and the `+` `SideTab`;
  the walk must consider only delegates (those with an `index` property).
- The menu `Repeater`: if the installed Qt Quick Controls does not parent
  `Repeater`-made items into the `Menu`'s content, switch to `Instantiator` with
  `onObjectAdded: plusMenu.insertItem(index, object)`. Checked at inspect.
- No back end: none of this talks to `rusty-mcp`. Theme: stock controls, no colour named.
- Keyboard: Ctrl+T and Ctrl+Shift+T unchanged; Ctrl+Shift+PgUp/PgDown unchanged. The
  menu is a mouse path added beside them, not a replacement.

### CodeGraph

Not applicable: no Rust symbols. QML by hand, per CONSTITUTION §18.

## Phase 3: Implement

Two files, as the manifest said; `qmllint -I /usr/lib/qt6/qml` exit 0 on both. Inspect
ran before the fast gate this time (AAR-021's lesson).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | the drop walk over `tabRow.children` might count the `Repeater`, the spacer or the `+` | rejected: only delegates declare `index`; `SideTab` has none (read), nor does `Repeater` or `Item` |
| F2 | correctness | delegates might not be in `tabRow.children` | rejected: a `Repeater` parents its delegates into its own parent, as siblings after it |
| F3 | correctness | `centroid.position` at release | rejected: the same read the Tasks page makes at `active` falling, and it works there |
| F4 | correctness | a vertical wobble might activate the drag | rejected as a defect: `yAxis.enabled: false`; and a drag past threshold cancels the tap either way (REQ-006) |
| F5 | correctness | a right- or middle-button press might start a drag | rejected: `DragHandler` accepts the left button by default |
| F6 | correctness | `insertItem(index + 2, …)` might land agents in the wrong place | rejected: static children exist before the `Instantiator` completes, and the index is explicit, so agents sit between the two separators |
| F7 | correctness | `required property string modelData` over a JS array model | rejected: Qt 6 provides `modelData` for array models in an `Instantiator` |
| F8 | keyboard first | the `+`'s tooltip lost "(Ctrl+T)" when the click became a menu, so the surface no longer answered the keyboard question | **confirmed**; the tooltip now names both keys (Ctrl+T page, Ctrl+Shift+T terminal), both unchanged in behaviour |
| F9 | prose | the menu's comment said "on either side of" the switcher; the agents come after it | **confirmed**; reworded |
| F10 | correctness | the `plus` scene pops the menu before layout | rejected: scenes run after the shot delay |
| F11 | correctness | `sp.mapToItem(null, mouse.x, 0)` maps from the right item | rejected: the `MouseArea` fills `sp`, so `mouse.x` is in `sp`'s frame |
| F12 | data safety | anything new stored | no finding: `moveTab` already saves the order; nothing else persists |
| F13 | theme | the `Menu`, the `ColumnLayout` | no finding: stock controls, no colour named |

CodeGraph: not applicable, no Rust symbols.
| F14 | validation evidence | the offscreen `plus` scene showed the agent items truncated ("Open Claude Code i…"): the stock `Menu` is narrow and the palette's context-free phrasing is redundant under the tab strip | **confirmed**; the items carry the agent's name alone, the way the top bar's glyph tooltip does |

## Phase 4: Validate

- `bin/gate.sh --fast` after inspect: `GATE GREEN [fast]` (fmt, clippy, test all ok).
- `cargo build -p rusty-app -p rusty-mcp` (the gate's `cargo test` does not produce the
  binary; the QML is compiled in), then `scripts/screenshot.sh <scratch> rename plus`
  against a scratch vault, offscreen, `RUSTY_DEBUG=1`. Both logs clean of
  `error|warning|TypeError|ReferenceError|Cannot assign|is not a type`.
  - `rename.png`: the Rename tab dialog with the field inside its width, Cancel and OK
    below it — REQ-003. Before this change the field ran past the right edge.
  - `plus.png`, first shot: the menu open under the `+` with the switcher, the agents
    and the custom terminal in order — REQ-004 — and the agent items truncated
    ("Open Claude Code i…"). That became F14; the second shot after the fix reads
    "Note or page…", "Claude Code", "Codex", "Gemini", "OpenCode", "Shell",
    "Custom terminal…" with nothing cut.
- `bin/gate.sh --diff` after F14 (the last gated edit): every step ok, `GATE GREEN
  [diff]`, receipt written.
- One line in the plain build is not this change's: `warning: the gold linker is
  deprecated and has known bugs with Rust`. It comes from the box's linker
  configuration, appears only in `cargo build` (clippy under `-D warnings` passed twice),
  and no Rust was touched. Noted, not acted on; it belongs to the dev-box handbook.
- REQ-001, REQ-002 and REQ-006 (the drags) rest on the readings in the ledger and on
  Chad's own use, per bulletin 2.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-006 satisfied — REQ-003 and REQ-004 by the offscreen
  scenes, the drags by the ledger's readings and Chad's use, REQ-005 by the sentence. None
  split, none waived.
- Wiki: run `9dfa6cb4-28cb-4e93-953b-bb372b2d8189`, `openwiki_finish` → `complete`; the
  Tabs bullet and a new splitter bullet on `workspace-app.md`, five claims added. The
  PostToolUse hook did not fire (fourth sighting); bulletin 3's recovery fed the genuine
  result to the hook script with the pair under `active/`.
- ROADMAP ticked under M8. `BF-rusty-moving-frame-delta-001` in the register and the AAR.
  Brain: timeline entry on `projects/rusty-v3`.
