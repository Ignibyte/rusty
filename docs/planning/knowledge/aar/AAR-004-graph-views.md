---
title: AAR-004-graph-views
pipeline_id: 9e4b7f12-3c8a-4d6e-b1f5-7a2c9d0e4f63
ticket: TICKET-004
submitted: 2026-09-03
---

# AAR-004-graph-views

## Recall log

- Register: `AD-rusty-tags-one-index-001`, `AD-rusty-lenient-pages-001`,
  `PR-rusty-qml-component-scope-001`, `PR-rusty-workspace-state-in-json-001`.
- Completed notes: TICKET-002 (tab kinds, state, screenshots), TICKET-003 (tags).
- Wiki: `workspace-app.md`, `vault-and-brain.md`. Brain: `projects/rusty-v3`.

## 1. Outcomes

- REQ-001 PASS, REQ-002 PASS, REQ-003 PASS, REQ-004 PASS, REQ-005 PASS, REQ-006 PASS.
  Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The index already had everything the graph needs; the tool is one query per table
  and a breadth-first walk, and its test covers the four shapes.
- The force layout in QML JavaScript settles a few hundred nodes in a second or two and
  reads well against Obsidian's screenshot on the first render.

## 3. What went poorly

- A signal named `settingsChanged` next to `property var settings` stopped the whole
  window from loading; the journal path from AAR-002 found it in one look.
- The screenshot script's fixture and its cleanup trap had two latent faults that a
  new folder exposed together.

## 4. Surprises

- `kill "${var:-0}"` in an EXIT trap is `kill 0` when the variable is unset, and it
  takes the calling shell with it.

## 5. Lessons

- PR-rusty-qml-signal-names-001: never declare a signal `<name>Changed` beside
  `property <name>`; the property already owns that signal.
- PR-rusty-never-kill-zero-001: a cleanup trap kills a pid only when it holds one;
  `${var:-0}` is `kill 0`.
- AD-rusty-graph-in-the-app-001: the graph's data is one tool (`brain_graph`); the
  layout, interaction and settings live in the app on a canvas, with the settings in
  the workspace state.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.5 h |
| 2 Design | 0.5 h | 0.5 h |
| 3 Implement | 2.5 h | 2 h |
| 3.5 Inspect | 0.5 h | 0.5 h |
| 4 Validate | 0.5 h | 1 h (the script faults) |
| 5 Complete | 0.5 h | 0.5 h |
