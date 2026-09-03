---
title: AAR-011-top-bar-command-and-agents
pipeline_id: 1213afdc-7c3e-4e41-b86d-1b2b7c55694f
ticket: TICKET-011
submitted: 2026-09-03
---

# AAR-011-top-bar-command-and-agents

## Recall log

- Register: the QML scope and signal-name rules, the build-before-shots rule, the lazy
  pane rule. TICKET-008 notes: the strip and its accepted `hyprctl` cost. Wiki:
  `workspace-app.md`. Code: `TopBar.qml`, the ribbon in `Main.qml`, `desk.rs`.

## 1. Outcomes

- REQ-001 to REQ-005 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The window already had every function the bar needed (`openTerminal`, `showRight`,
  the glyph and name maps), so the bar stayed stateless and the change was three signals
  and a delegate.
- The screenshot script proved the layout in one run, and CodeGraph confirmed the strip
  was the only Hyprland-aware element, so `Desk` could lose the whole reading.

## 3. What went poorly

- The doc comment kept the word `hyprctl` after the code lost it, which would have made
  the requirement's grep count a comment; caught at the fast gate.

## 4. Surprises

- None. The ticket was as small as it looked.

## 5. Lessons

- No new register entries. The rules that applied (`PR-rusty-qml-component-scope-001`,
  `PR-rusty-build-workspace-before-shots-001`) held.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.3 h | 0.2 h |
| 2 Design | 0.3 h | 0.2 h |
| 3 Implement | 0.5 h | 0.3 h |
| 3.5 Inspect | 0.2 h | 0.1 h |
| 4 Validate | 0.3 h | 0.3 h |
| 5 Complete | 0.3 h | 0.3 h |
