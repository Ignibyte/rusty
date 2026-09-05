---
title: Terminal clipboard — notes
pipeline: 7034f494-16ee-4d43-bd21-8353ce4291f4
ticket: TICKET-021
---

# Terminal clipboard: notes

## Recall (2026-09-05)

- Bulletins: no critical bulletin. Bulletin 2 applies to validation — Chad uses this box
  while agents work on it, so no synthetic keystrokes and no workspace switching for
  screenshots. That shapes how REQ-001 to REQ-006 are verified: by reading the component
  and the widget's exposed surface, and by Chad's own use, not by driving his desktop.
- Knowledge register, the two entries that bear on this file:
  - `PR-rusty-signals-through-connections-001` — attach handlers for a third-party type's
    signals through `Connections { ignoreUnknownSignals: true }`; an `onFoo` property for
    a signal the type lacks fails the whole component load **silently**. This is why
    `copyAvailable` is read through `Connections` (locked decision 5).
  - `AD-rusty-workspace-is-obsidian-001` — workspace keys stand down while a terminal has
    focus (`Main.qml:111`). A window-level `Shortcut` therefore cannot serve REQ-001 or
    REQ-002; the keys must attach to the terminal item itself (locked decision 2).
  - `PR-rusty-lazy-pane-terminals-001` — a terminal starts its tmux session when first
    shown. Nothing here changes that, but the context menu must not force the component
    to load in a pane that has not been shown.
- Wiki: `openwiki/workspace-app.md` describes the terminals as tmux sessions with titles
  forwarded and a bell that notifies. No clipboard is documented, because there is none.
- Nearest completed notes: `knowledge-workspace-shell` (which built the terminal
  component) and `session-resilience` (which owns the tmux session lifetime).
- Brain: `AD-rusty-agents-are-terminals-001` — agents are terminals, there is no
  in-process chat. That decision is what makes this ticket necessary rather than
  incidental: the terminal is the interface, so its clipboard is the product's clipboard.

## The widget's actual surface

Read from `/usr/lib/qt6/qml/QMLTermWidget/libqmltermwidget.so` rather than assumed, since
QMLTermWidget ships no `.qmltypes`:

```
copyClipboard  pasteClipboard  pasteSelection  copyAvailable
selectionChanged  textSelected  isBusySelecting  search  clearScreen
bracketedPasteMode / setBracketedPasteMode
```

`nm -D` confirms `Konsole::TerminalDisplay::copyClipboard()` and `::pasteClipboard()` are
defined and exported. Three findings that changed the design:

1. `pasteSelection` exists, so middle click can be the primary selection rather than a
   second clipboard paste (locked decision 3).
2. `copyAvailable(bool)` exists, so the menu's Copy can be enabled by real selection
   state rather than always enabled (REQ-004).
3. `bracketedPasteMode` exists and is already handled by the widget, so a paste into a
   shell that asked for bracketed paste is wrapped by the widget itself. Nothing to do,
   but worth knowing before adding a paste path.

## Phase 2: Design

### Architecture and data flow

One QML component, no Rust, no tool, no store. Every path ends in a call the widget
already exports:

```
Ctrl+Shift+C ─┐
 menu Copy   ─┴─▶ term.copyClipboard()
Ctrl+Shift+V ─┐
 menu Paste  ─┴─▶ term.pasteClipboard()
 middle click ──▶ term.pasteSelection()        (the primary selection)
 term.copyAvailable(bool) ──▶ tab.copyAvailable ──▶ menu Copy.enabled
```

The keys attach to the terminal item with `Keys.priority: Keys.BeforeItem`, so the
attached handler sees the event before the widget's own key handling and can accept it;
every other key falls through untouched, which is what keeps Ctrl+C an interrupt.

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/qml/AgentTerminal.qml` | `import QtQuick.Controls`; a `copyAvailable` property; `Keys` on the widget; a `MouseArea` accepting only middle and right; a two-item `Menu`; one more handler in the existing `Connections` on `term` |

Nothing else. The right pane and the terminal tabs both instantiate this component, so
both get the behaviour from the one edit.

### Store, tools, compatibility

None. No schema, no tool, no setting.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 copy on Ctrl+Shift+C | reading: the handler calls `copyClipboard()` on `Key_C` with Control and Shift; smoke by Chad |
| REQ-002 paste on Ctrl+Shift+V | reading: `pasteClipboard()` on `Key_V`; smoke by Chad |
| REQ-003 middle click pastes primary | reading: `pasteSelection()` on `Qt.MiddleButton`; smoke by Chad |
| REQ-004 menu, Copy gated on selection | reading: `Menu` with `Copy.enabled: tab.copyAvailable`, fed by `onCopyAvailable`; smoke by Chad |
| REQ-005 Ctrl+C stays the interrupt | reading: the handler returns early unless both Control and Shift are down, so Ctrl+C alone is never accepted; smoke by Chad |
| REQ-006 left drag still selects | reading: `acceptedButtons: Qt.MiddleButton \| Qt.RightButton`, so a left press is not accepted and propagates to the widget; smoke by Chad |

The QML test that refuses literal `pixelSize` (`theme.rs`) runs in the gate and covers
the new `Menu` for the text-size rule.

### Risks

- **Key priority.** If the widget consumed Ctrl+Shift+C before the attached handler, the
  chord would do nothing. `Keys.BeforeItem` is the documented way to win that ordering.
- **Stealing the left button.** A `MouseArea` over the terminal that accepted the left
  button would end selection by drag. Locked decision 4 restricts it to middle and right.
- **The lazy-pane rule.** The `Menu` is instantiated with the component but opens nothing
  and starts nothing; the tmux session still starts on first show.
- **Theme.** The `Menu` is a stock control, the same as `tabMenu` in `Main.qml`; no colour
  is named.
- **No back end.** Not involved; the terminal does not talk to `rusty-mcp`.
- **Wayland primary selection.** `pasteSelection` reads the primary selection through
  Qt; under a compositor without `primary-selection`, it pastes nothing and does no harm.

### CodeGraph

Not applicable: no Rust symbol changes. QML inspected directly, per CONSTITUTION §18.

### Decisions and alternatives

Recorded on the spec as locked decisions 1 to 5.

## Phase 3: Implement

One file, as the manifest said. `qmllint -I /usr/lib/qt6/qml` on the component: exit 0,
no output. The Phase 3 `bin/gate.sh --fast` ran against the first version of the diff
(the `MouseArea` one); finding F5 below changed the file after it started, so the
`--diff` run in Phase 4 is the gate that covers the shipped code.

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | `event.modifiers & Qt.ControlModifier` on an int against an enum | rejected: QML ints, bitwise `&` is correct; the `&&` of two masks is truthy only when both are set |
| F2 | correctness | Shift+C might report a different key code | rejected: `Qt.Key_C` is one value for both cases |
| F3 | correctness | the button surface also covers the scrollbar child | rejected: the scrollbar uses neither middle nor right |
| F4 | correctness | a `MouseArea` with no `onWheel` might still swallow wheel events and kill scrollback | **confirmed as a risk**, resolved with F5 |
| F5 | correctness | **Konsole's `TerminalDisplay` handles the middle button natively** (the exported `pasteSelection` slot is that path), so a `MouseArea` above it could paste twice on one click | **confirmed**; fixed: `TapHandler` with `gesturePolicy: ReleaseWithinBounds`, which takes an exclusive grab on press so the widget never sees the middle click, and which never handles wheel |
| F6 | correctness | `termMenu.popup()` position | rejected: no-arg `popup()` opens at the cursor |
| F7 | complexity | `Keys.priority: Keys.BeforeItem` is the default, so the line is redundant | rejected as a defect; kept as stated intent, since the ordering is the whole reason the chords work here |
| F8 | data safety | nothing touches the store, the disk or the network | no finding |
| F9 | keyboard first | the chords are the keyboard path; the menu is the mouse path | no finding |
| F10 | theme | a stock `Menu`, as `tabMenu` in `Main.qml`; no colour named | no finding |
| F11 | third-party signals | `onCopyAvailable` is inside the existing `Connections { ignoreUnknownSignals: true }` | no finding; `PR-rusty-signals-through-connections-001` honoured |
| F12 | prose | comments and the two menu strings | no finding |

CodeGraph: not applicable, no Rust symbols changed.

Post-fix `qmllint`: exit 0.

## Phase 4: Validate

### The gate, on the shipped file

`bin/gate.sh --diff` after the F5 fix (the last gated edit), 2026-09-05 (box time
2026-09-04 evening). Step lines as printed:

```
-- test ok == doc -- doc ok == shell-syntax -- shell-syntax ok == secrets -- secrets ok == whitespace -- whitespace ok receipt written: .git/rusty-gate-receipt GATE GREEN [diff] 
```

Receipt written to `.git/rusty-gate-receipt`. No Rust changed, so the test set is the
existing one; the QML literal-`pixelSize` test in `theme.rs` covers the new `Menu`.

### Runtime load

The QML module is compiled into the binary (`qrc:/qt/qml/dev/ignibyte/rusty/qml/Main.qml`,
`build.rs` via `cxx_qt_build`), and the gate's `cargo test` does not produce
`target/debug/rusty` — it was still the 21:25 build against a 22:14 edit. So a
`cargo build -p rusty-app -p rusty-mcp` precedes the offscreen check below.

`cargo build -p rusty-app -p rusty-mcp` finished 22:16:37 (the QML edit is 22:14:57).
`scripts/screenshot.sh <scratch> right:agent` against a scratch vault and a scratch
`rusty-mcp` on its own port, offscreen (`QT_QPA_PLATFORM=offscreen`, `RUSTY_DEBUG=1`):

- `right-agent.log`: no line matching `AgentTerminal`, `error`, `warning`, `TypeError`,
  `ReferenceError`, `Cannot assign`, `is not a type` or `Unable to`. The component loaded.
- Two instances ran and forwarded titles — `rusty: title of rusty-shot-shell …` (the
  terminal tab) and `rusty: title of rusty-pane-shell …` (the agent pane) — so both
  tmux sessions started and the `Connections` on `term` and `termSession` still bind
  with the new handler in place.
- `right-agent.png` written, 207 KB, the pane showing a live shell prompt.

The scratch sessions are named `rusty-shot-*` and `rusty-pane-*` and the script kills
them on exit; nothing of Chad's was touched, per bulletin 2. Input (the chords, the
clicks) is not driven synthetically for the same reason; REQ-001 to REQ-006 rest on the
reading in the regression table and on Chad's own use.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-006 satisfied — each with its reading in the Phase 2
  regression table, the offscreen load of both instances in Phase 4, and Chad's own use
  as the input check bulletin 2 leaves to him. None split, none waived.
- Wiki: run `a6754cbc-596d-40d7-9e32-59ef454d9527`, `openwiki_finish` → `complete`;
  `workspace-app.md`'s terminal bullet documents the clipboard, two claims re-anchored,
  one added. The PostToolUse hook did not fire (third sighting); bulletin 3's recovery
  fed the genuine result to the hook script with the pair under `active/`.
- ROADMAP ticked under M8. `PR-rusty-exclusive-grab-over-native-handlers-001` in the AAR
  and the register. Brain: timeline entry on `projects/rusty-v3`.
