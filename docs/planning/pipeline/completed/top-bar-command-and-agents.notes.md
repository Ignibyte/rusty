---
title: Top bar, command and agents: notes
pipeline_id: 1213afdc-7c3e-4e41-b86d-1b2b7c55694f
---

# Top bar, command and agents: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (two notices). Register: `PR-rusty-qml-component-scope-001` (bind
  shared objects through the window), `PR-rusty-qml-signal-names-001` (no
  `<prop>Changed` signals), `PR-rusty-build-workspace-before-shots-001`,
  `PR-rusty-lazy-pane-terminals-001` (the agent pane starts its session when first
  shown). Completed notes: TICKET-008 (the top bar and the strip, inspect finding 6
  accepted two `hyprctl` calls every two seconds). Wiki: `workspace-app.md` lines 75 and
  90 describe `Desk`'s workspace reading and the ribbon's contents. Code read:
  `TopBar.qml` (87 lines: brand, the strip as a `Repeater` over `desk.workspaces`, the
  vault state, memory, CPU, clock, quit), the ribbon in `Main.qml` (`RibbonButton`, the
  command button at line 738, the agents `Repeater` at 745 to 756), `desk.rs`
  (`workspaces`, `active_workspace`, `hyprland`, `switch_workspace`,
  `hyprland_workspaces`), the Settings hint at `SettingsPage.qml:86`. `Main.qml` already
  has `agents`, `agentGlyphs`, `agentNames`, `openTerminal` and `showRight("agent")`
  with `rightPane.program`.
- Decisions: the three locked decisions in the spec.

## Phase 2: Design

- Architecture and data flow: `TopBar` stays stateless. It takes `agents`, `agentGlyphs`
  and `agentNames` from the window as properties and raises three signals,
  `commandRequested`, `agentRequested(program)` and `agentPaneRequested(program)`; the
  window answers with `palette.show()`, `openTerminal(program, "", "", "")` and
  `rightPane.program = program; showRight("agent")`, the same paths the ribbon and the
  palette use. A `BarButton` component (an `Icon` or a glyph at 12 px, a hover frame, a
  tooltip, a left and a right `TapHandler`) replaces the strip's delegate. `Desk` keeps
  memory, CPU, clock and the login name and loses `workspaces`, `active_workspace`,
  `hyprland`, `switch_workspace` and `hyprland_workspaces`; the two-second timer stays.
- File manifest:
  - `crates/rusty-app/qml/TopBar.qml`: the strip out; the command button and the agent
    glyphs in; the properties and signals above.
  - `crates/rusty-app/qml/Main.qml`: the `TopBar` instance wired; the ribbon loses the
    command button, the agents `Repeater` and the separator before it.
  - `crates/rusty-app/src/desk.rs`: the workspace reading out; the doc comment says why.
  - `crates/rusty-app/qml/SettingsPage.qml`: the hint names the right-click.
  - Phase 5: `README.md` (the ribbon list, the top bar sentence), `ROADMAP.md`, the wiki
    page `workspace-app.md` (lines 75 and 90).
- Store consequences: none.
- Tool contract: none.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | the `reading` screenshot: brand, command button, agent glyphs, no strip; the bar's model is `win.agents` from `terminals.programs()` |
  | REQ-002 | the handlers by reading; the `right:agent` screenshot shows the pane the right-click opens |
  | REQ-003 | the `reading` screenshot (ribbon without command or agents); the `palette` screenshot; the `Ctrl+P` `Shortcut` unchanged by reading |
  | REQ-004 | `grep -c hyprctl crates/rusty-app/src/desk.rs` is 0; `cargo test -p rusty-app` keeps the readings test green |
  | REQ-005 | the Settings text by reading |
- Risks: data safety, none. Keyboard: Ctrl+P and every ribbon key unchanged; the bar's
  buttons are pointer-only, as the ribbon's were, and the palette lists the same actions
  with keys. Theme: every colour a token. The right-click on an agent sets the pane's
  program; when the pane's terminal already runs another agent it keeps its session, the
  same limit the pane's own selector has today. Off Hyprland nothing changes, since the
  strip was the only Hyprland-aware element.
- CodeGraph evidence: `Desk` (`desk.rs`) has one QML user, `Main.qml` (the instance and
  the timer) and `TopBar.qml`; `hyprland_workspaces` has one caller, `take_readings`;
  `switch_workspace` is called only from the strip's `TapHandler`. Removing them touches
  nothing else. `installed_agents` and `programs()` are unchanged.

## Phase 3: Implement

- Built: `TopBar.qml` rewritten around a `BarButton` component (icon or glyph at 12 px,
  hover frame, tooltip, left and right `TapHandler`s): the command button, then one glyph
  per program in `agents`, the strip gone; `Main.qml` wires `agents`, `agentGlyphs`,
  `agentNames` and the three signals into the `TopBar` instance and drops the ribbon's
  command button, its agents `Repeater` and the separator before it; `desk.rs` loses the
  workspace properties, `switch_workspace`, `hyprland_workspaces` and the `Command`
  import; the Settings hint names the right-click.
- Deviations: the shell keeps its `$` glyph in the bar, as it had in the ribbon; the doc
  comment in `desk.rs` was reworded so the requirement's `grep hyprctl` counts code only.
- Fast gate: `cargo build --workspace` clean (11.75 s, the gold-linker warning predates
  this ticket); `bin/gate.sh --fast` on 2026-09-03: `GATE GREEN [fast]`.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | a right-click on an agent while the pane's terminal already runs another agent sets `rightPane.program` but the running session stays | low | accepted: the pane's own selector has the same limit today; noted in the risks |
| 2 | keyboard | the bar's buttons are pointer-only | ok | every action stays in the palette with its key (Ctrl+P, and "Terminal: Open <agent>"), as the ribbon's were |
| 3 | theme | every colour in the new QML is a token (`panel2`, `panel3`, `line`, `lineBright`, `accent`, `muted`, `gold`, `alive`, `red`, `bright`) | ok | by reading |
| 4 | scope | `TopBar` reads `agentGlyphs[modelData]` on a `var` map from the window rather than an id | ok | PR-rusty-qml-component-scope-001 honoured; the window passes the maps |
| 5 | correctness | `Desk` off Hyprland: nothing was Hyprland-aware but the strip, so removing `hyprland` changes no other path | ok | by reading `Main.qml` (`desk.user` is the only other use) |
| 6 | prose | the Settings hint, the QML comments, the doc comment | ok | zero em dashes; no banned words |
| 7 | evidence | the `right:agent` scene shows a scratch tmux session whose status line carries the box's host name | low | the scene stays out of the docs; the docs screenshots use `reading` and `palette` only |

- Post-implementation CodeGraph: `codegraph_explore` over `Desk`, `DeskRust`,
  `take_readings`, `refresh`: `take_readings` has two callers (`Default`, `refresh`),
  `DeskRust` one; no symbol named `hyprland_workspaces` or `switch_workspace` remains;
  the QML users of `Desk` are the timer in `Main.qml` and `TopBar.qml`.

## Phase 4: Validate

- Tests run (commands and output): the gate ran `cargo test --workspace` (every test
  green, the readings test in `desk.rs` included); `grep -c hyprctl
  crates/rusty-app/src/desk.rs` prints 0; `grep -c 'model: win.agents'` and `grep -c
  'label: "cmd"'` over `Main.qml` print 0; `scripts/screenshot.sh <dir> reading palette
  right:agent` wrote the three scenes with no QML error.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: `GATE GREEN [diff]`, `receipt written:
  .git/rusty-gate-receipt`.
- Smoke evidence: the `reading` scene shows the bar as brand, command icon, the glyphs
  for the five programs on the scratch machine, no strip, and the ribbon as new, daily,
  graph, tasks, memory, skills, secrets, setup; the `palette` scene shows the palette open
  over it; the `right:agent` scene shows the agent pane the right-click opens. No
  synthetic input on the desktop.
- Skips or pre-existing failures: the click and right-click handlers are verified by
  reading (`onAgentRequested` to `openTerminal`, `onAgentPaneRequested` to
  `rightPane.program` and `showRight("agent")`); the gold-linker warning predates the
  ticket.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the `reading` scene; the bar's model is
  `terminals.programs()`, covered by the tabs tests); REQ-002 PASS (the handlers by
  reading; the `right:agent` scene); REQ-003 PASS (the `reading` and `palette` scenes;
  the `Shortcut` unchanged); REQ-004 PASS (`grep -c hyprctl` is 0; the readings test);
  REQ-005 PASS (the Settings hint by reading).
- Docs: `README.md` (the workspace paragraph, the chrome sentence), `ROADMAP.md` (M7
  line), the ten docs screenshots re-rendered from the scratch vault, since every one
  shows the top bar.
- Wiki: `update` run `e8d74502` through the lifecycle: the top-bar claim on
  `workspace-app.md` updated with fresh evidence, the page's `Desk` and layout bullets
  rewritten; `openwiki_finish` returned `status: complete`.
- AAR: `docs/planning/knowledge/aar/AAR-011-top-bar-command-and-agents.md`; no new
  register IDs.
- Brain capture: timeline entry on `projects/rusty-v3` at delivery.
- Archive: this pair lives in `completed/`; the ticket in `closed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
