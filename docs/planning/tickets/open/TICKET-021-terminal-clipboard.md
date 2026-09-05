---
title: TICKET-021-terminal-clipboard
status: open
ticket_number: 021
type: fix
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/terminal-clipboard.spec.md
---

# TICKET-021-terminal-clipboard

## Summary

Copy and paste in the agent terminals: `Ctrl+Shift+C` / `Ctrl+Shift+V`, middle-click paste, and a right-click menu.

## Why

There is no way to get text in or out of a terminal tab. `AgentTerminal.qml` wires no clipboard at all — no `Keys`, no shortcuts, no context menu — so an agent's answer cannot be copied and a path cannot be pasted in. Chad assumed it was him; it is not.

The widget already exposes what is needed: `libqmltermwidget.so` registers `copyClipboard` and `pasteClipboard` as QML-callable, alongside a `selectionChanged` signal. Nothing calls them.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `Ctrl+Shift+C` is pressed in a terminal with a selection, the terminal shall copy the selection to the clipboard. | smoke on the box |
| REQ-002 | WHEN `Ctrl+Shift+V` is pressed in a terminal, the terminal shall paste the clipboard at the cursor. | smoke |
| REQ-003 | WHEN the middle mouse button is clicked in a terminal, the terminal shall paste the primary selection. | smoke |
| REQ-004 | WHEN a terminal is right-clicked, a menu shall offer Copy (enabled only with a selection) and Paste. | smoke; screenshot |
| REQ-005 | WHEN a terminal has focus, `Ctrl+C` shall still reach the shell as an interrupt rather than being taken as copy. | smoke: interrupt a running command |

## Scope

- In: `AgentTerminal.qml`; the same bindings for the right pane's terminal.
- Out: replacing the terminal widget; embedding a real Alacritty (not practical for a Wayland client inside a QML window, and the widget is already themed to match); scrollback search.

## Notes

- REQ-005 is the trap: bind the shifted chords only, and let unshifted keys through.
- Pipeline spec: TBC.
