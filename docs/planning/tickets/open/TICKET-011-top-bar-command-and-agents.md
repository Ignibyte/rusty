---
title: TICKET-011-top-bar-command-and-agents
status: open
ticket_number: 011
type: feature
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/top-bar-command-and-agents.spec.md
---

# TICKET-011-top-bar-command-and-agents

## Summary

The top bar drops the Hyprland workspace strip and takes the command button and one small
glyph per agent CLI found on the machine; the ribbon loses both.

## Why

The strip in `TopBar.qml` draws Hyprland's workspaces with the active one lit, polling
`hyprctl` through `Desk` to do it, which is the strip waybar already shows. The mock meant the
agents to sit in the top bar, and the Settings page still says "click an agent in the top bar",
while the code puts them in the ribbon. Chad wants the command palette and the agents up there
as very small icons, and the duplicate switcher gone.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the window is shown, the top bar shall show, left to right, the brand, a command button that opens the palette, and one glyph per agent CLI found on the machine, and shall not show the workspace strip. | screenshot; test on the bar's model |
| REQ-002 | WHEN an agent glyph is clicked, the app shall open that agent in a new tab; WHEN it is right-clicked, the app shall open it in the agent pane beside the note. | smoke |
| REQ-003 | WHEN the ribbon is shown, it shall hold neither the command button nor the agent buttons, and Ctrl+P shall still open the palette. | screenshot; hotkey test |
| REQ-004 | WHEN the app runs, `Desk` shall not poll Hyprland's workspaces; memory, CPU and the clock stay. | test on `Desk` |
| REQ-005 | WHEN Settings describes the agents, its text shall match where they are. | review |

## Scope

- In: `TopBar.qml`, the ribbon in `Main.qml`, the workspace reading in `desk.rs`, agent glyphs at the top bar's size, the Settings hint.
- Out: the ribbon redo (TICKET-017); new agent CLIs.

## Notes

- Pipeline spec: docs/planning/pipeline/active/top-bar-command-and-agents.spec.md
- Related docs: `docs/architecture.md` (as built: the workspace), `crates/rusty-app/qml/TopBar.qml`, `crates/rusty-app/src/desk.rs`.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "command should probably move up into the very top bar along with the claude, codex, gemini in very small icons. lets replace the omarchy window switcher since the os already has some with the command/ai".
- Follow-ups opened: none.
