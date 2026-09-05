---
title: Terminal clipboard
pipeline_id: 7034f494-16ee-4d43-bd21-8353ce4291f4
status: Phase 5 — Complete PASS
ticket: TICKET-021
ticket_doc: docs/planning/tickets/open/TICKET-021-terminal-clipboard.md
aar: docs/planning/knowledge/aar/AAR-021-terminal-clipboard.md
sealed:
created: 2026-09-05
---

# Terminal clipboard: spec

## Intent

Text cannot get in or out of an agent terminal. `AgentTerminal.qml` binds no clipboard
at all, so an agent's answer cannot be copied and a path cannot be pasted in. Chad,
2026-09-04: "i cant figure out how to copy or paste inside here. maybe thats me". It was
not him.

The widget already has the surface. `libqmltermwidget.so` exposes `copyClipboard`,
`pasteClipboard`, `pasteSelection` and a `copyAvailable(bool)` signal to QML. Nothing
calls them. This pipeline binds them and nothing else.

## Scope

- In: `crates/rusty-app/qml/AgentTerminal.qml` — the two chords, middle-click paste from
  the primary selection, a context menu whose Copy follows `copyAvailable`. The same
  component serves terminal tabs and the right pane, so both are covered by one change.
- Out (named seams, not forgotten): replacing the terminal widget; embedding Alacritty
  (not practical for a Wayland client inside a QML window, and the widget is already
  themed to match); scrollback search, which `search` would serve when someone wants it;
  a paste confirmation for multi-line clipboard content.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN Ctrl+Shift+C is pressed in a terminal, the terminal shall copy the selection to the clipboard. | smoke on the box |
| REQ-002 | WHEN Ctrl+Shift+V is pressed in a terminal, the terminal shall paste the clipboard at the cursor. | smoke |
| REQ-003 | WHEN the middle mouse button is clicked in a terminal, the terminal shall paste the primary selection. | smoke |
| REQ-004 | WHEN a terminal is right-clicked, a menu shall offer Copy and Paste, Copy enabled only while a selection exists. | smoke; screenshot |
| REQ-005 | WHEN Ctrl+C is pressed in a terminal, it shall reach the shell as an interrupt rather than being taken as copy. | smoke: interrupt a running command |
| REQ-006 | WHEN the left mouse button is used in a terminal, selection by drag shall behave as it did before this change. | smoke |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The chords are Ctrl+Shift+C and Ctrl+Shift+V | The terminal convention, and the only pair that leaves Ctrl+C as the interrupt (REQ-005) | Ctrl+C/Ctrl+V, which would break the interrupt; a modifier-free menu only |
| 2 | Keys attach to the terminal item with `Keys.priority: Keys.BeforeItem` | Workspace shortcuts stand down while a terminal has focus (`AD-rusty-workspace-is-obsidian-001`), so a global `Shortcut` would never fire here | A window-level `Shortcut`; a wrapper Item with `forwardTo` |
| 3 | Middle click uses `pasteSelection`, not `pasteClipboard` | Middle click is the primary selection everywhere else on X11 and Wayland; pasting the clipboard would surprise | Middle click as a second Ctrl+Shift+V |
| 4 | The mouse area accepts only the middle and right buttons | Left-button events then reach the terminal untouched, so selection by drag is unaffected (REQ-006) | Filtering inside one all-button handler, which risks swallowing drags |
| 5 | `copyAvailable` is read through `Connections { ignoreUnknownSignals: true }` | `PR-rusty-signals-through-connections-001`: an `onFoo` handler for a signal a third-party type lacks fails the whole component load silently | An `onCopyAvailable` property directly on the widget |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-021-terminal-clipboard.md`
- Intake: none; reported from use
- Architecture: `openwiki/workspace-app.md` (the terminals), `AD-rusty-agents-are-terminals-001`
- Prevention rules in play: `PR-rusty-signals-through-connections-001`, `PR-rusty-lazy-pane-terminals-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; no seal needed (no new surface) |
| 2 Design | The manifest and the key path | design actionable |
| 3 Implement | The bindings | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger | confirmed findings resolved |
| 4 Validate | Smoke evidence, `bin/gate.sh --diff` green | receipt matches worktree |
| 5 Complete | Requirement audit, wiki, AAR, register, brain, archive | pair archived |
