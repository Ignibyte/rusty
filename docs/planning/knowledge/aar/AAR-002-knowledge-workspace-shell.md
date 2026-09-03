---
title: AAR-002-knowledge-workspace-shell
pipeline_id: 6b0c5d3e-2c0f-4b7e-9f0a-1d2e3f4a5b6c
ticket: TICKET-002
submitted: 2026-09-02
---

# AAR-002-knowledge-workspace-shell

## Recall log

- Register: `AD-rusty-files-are-the-truth-001`, `AD-rusty-vault-rules-001`,
  `PR-rusty-scope-the-sqlite-guard-001`, `PR-rusty-signals-through-connections-001`,
  `PR-rusty-probes-use-throwaway-rows-001` all bear on this work.
- Completed notes: AAR-001 (the workflow itself).
- Brain: the v3 design record page and the 2026-09-02 timeline entries.

## 1. Outcomes

- REQ-001 PASS, REQ-002 PASS, REQ-003 PASS, REQ-004 PASS, REQ-005 PASS, REQ-006 PASS,
  REQ-007 PASS, REQ-008 PASS, REQ-009 PASS. Evidence in the pipeline notes, Phase 5.

## 2. What went well

- The renderer was built and tested without Qt: every construct has a unit test, and the
  reading view came up right on the second screenshot.
- The scratch-vault screenshot script gave real evidence for every requirement without a
  keystroke on Chad's desktop, and it now makes the docs' screenshots.
- The eight tools were thin; the manager methods and the smoke test carried the behaviour.

## 3. What went poorly

- Two silent QML failures (an inline component binding to itself, a final property
  overridden) cost an hour because Qt's messages had gone to the journal, not the terminal.
- QtCore `Settings` lost string values during the run; the fix was to stop using it for
  the workspace state.

## 4. Surprises

- pulldown-cmark splits `[!kind]` into pieces, so callouts need a pre-pass.
- The offscreen platform never exposes a window, so `grabToImage` returns false; only
  `QQuickWindow::grabWindow` renders on demand.
- Inside a QML `Component` scope, an unqualified name finds the component's own property
  before an id of the enclosing document; the same pattern works outside a Component.
- A terminal pane that starts at load runs its agent even when the pane is hidden.

## 5. Lessons

- PR-rusty-qml-component-scope-001: inside an inline `Component`, bind shared objects
  through the window (`theme: win.theme`), never by bare id.
- PR-rusty-qt-logs-in-journal-001: when the app prints nothing, read
  `journalctl -t rusty` or set `QT_FORCE_STDERR_LOGGING=1`; Qt routes messages to journald
  when stderr is not a tty.
- PR-rusty-workspace-state-in-json-001: keep app state that is not the window geometry in
  the JSON files the Rust side owns; QtCore `Settings` rewrote string properties with their
  defaults.
- PR-rusty-offscreen-shots-grab-window-001: screenshots offscreen go through
  `QQuickWindow::grabWindow` (the `Tools` object) against a scratch vault.
- PR-rusty-lazy-pane-terminals-001: a terminal starts its session when first shown.
- AD-rusty-renderer-in-core-001: markdown is rendered to Qt rich text in `rusty-core`
  (`brain::render`), served by `brain_render`, colours inlined from a style the app sends.
- AD-rusty-workspace-is-obsidian-001: the app is laid out as Obsidian is; pages, agent
  terminals and built-in views are all tabs; the right sidebar holds an agent pane; keys
  are Obsidian's and stand down while a terminal has focus.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 1 h | 1 h |
| 2 Design | 1 h | 1 h |
| 3 Implement | 6 h | 5 h |
| 3.5 Inspect | 1 h | 1 h (folded into the screenshot rounds) |
| 4 Validate | 1 h | 1.5 h (the offscreen grab and the settings hunt) |
| 5 Complete | 0.5 h | 0.5 h |
