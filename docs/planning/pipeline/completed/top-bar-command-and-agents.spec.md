---
title: Top bar, command and agents
pipeline_id: 1213afdc-7c3e-4e41-b86d-1b2b7c55694f
status: Phase 5 — Complete PASS
ticket: TICKET-011
ticket_doc: docs/planning/tickets/open/TICKET-011-top-bar-command-and-agents.md
aar: docs/planning/knowledge/aar/AAR-011-top-bar-command-and-agents.md
sealed: not required (no new tab, store or dependency). Direction: Chad, 2026-09-03 15:40, "command should probably move up into the very top bar along with the claude, codex, gemini in very small icons. lets replace the omarchy window switcher since the os already has some with the command/ai"; 16:00, "lets start working on these" (both relayed by the rustal session)
created: 2026-09-03
---

# Top bar, command and agents: spec

## Intent

The top bar drops the Hyprland workspace strip, which duplicates waybar, and takes the
command button and one small glyph per agent CLI found on the machine; the ribbon loses
both. `Desk` stops polling `hyprctl`. The Settings hint that already says "click an
agent in the top bar" becomes true.

## Scope

- In: `TopBar.qml` (the strip out; the command button and the agent glyphs in, a click
  opening a tab and a right-click opening the agent pane), the ribbon in `Main.qml`
  (the command and agent buttons out), `desk.rs` (the workspace reading and
  `switch_workspace` out; memory, CPU, clock and the login name stay), the Settings
  hint, the docs and the wiki.
- Out (named seams, not forgotten): the ribbon redo (TICKET-017); new agent CLIs; the
  text-size setting (TICKET-012), so the new glyphs take the bar's current sizes.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the window is shown, the top bar shall show, left to right, the brand, a command button that opens the palette, and one glyph per agent CLI found on the machine, and shall not show the workspace strip. | screenshot; the bar's model is `terminals.programs()`, covered by the tabs tests |
| REQ-002 | WHEN an agent glyph is clicked, the app shall open that agent in a new tab; WHEN it is right-clicked, the app shall open it in the agent pane beside the note. | smoke by reading the handlers; screenshot of the pane scene |
| REQ-003 | WHEN the ribbon is shown, it shall hold neither the command button nor the agent buttons, and Ctrl+P shall still open the palette. | screenshot; the `Shortcut` by reading; the palette scene |
| REQ-004 | WHEN the app runs, `Desk` shall not poll Hyprland's workspaces; memory, CPU and the clock stay. | `cargo test -p rusty-app` (the readings test); `grep hyprctl` over `desk.rs` finds nothing |
| REQ-005 | WHEN Settings describes the agents, its text shall match where they are. | review |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The strip goes entirely, `Desk` loses its workspace properties and `switch_workspace`. | Waybar already shows the workspaces; two `hyprctl` calls every two seconds bought nothing. | Keeping the properties unused. |
| 2 | The agent glyphs are the same characters the tabs use (`agentGlyphs` in `Main.qml`), at the bar's micro size, with a tooltip naming the agent. | One glyph per agent everywhere; the bar is 33 px tall. | Icons from `Icon.qml` (none exist per agent). |
| 3 | Click opens a tab; right-click opens the agent pane, both through `Main.qml`'s existing functions, wired as signals from `TopBar`. | The bar owns no state (§14); the window already has `openTerminal` and `showRight`. | The bar calling into the window by id (PR-rusty-qml-component-scope-001). |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-011-top-bar-command-and-agents.md`
- Intake: none
- Design references: `crates/rusty-app/qml/TopBar.qml`, the ribbon in
  `crates/rusty-app/qml/Main.qml`, `crates/rusty-app/src/desk.rs`, the mock
  `docs/design/rusty-omarchy.html` (the assistant in the top bar)
- Architecture: `docs/architecture.md` (the skinned bullet: `Desk` reads the machine for
  the top bar)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
