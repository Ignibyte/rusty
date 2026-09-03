---
title: AAR-008-amber-phosphor-and-themes
pipeline_id: c2bb8a6f-7da2-489c-87ed-d30218b1b8fe
ticket: TICKET-008
submitted: 2026-09-03
---

# AAR-008-amber-phosphor-and-themes

## Recall log

- Register: `AD-rusty-workspace-is-obsidian-001`, `PR-rusty-workspace-state-in-json-001`,
  `PR-rusty-qml-component-scope-001`, `PR-rusty-qml-signal-names-001`.
- Completed notes: TICKET-002 (tokens), TICKET-004 (panel styling).
- Wiki: `workspace-app.md`, `markdown-rendering.md`. Brain: `projects/rusty-v3`.

## 1. Outcomes

- REQ-001 to REQ-008 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- Keeping the older token names in `skin::tokens` meant the whole restyle was
  additive: no binding broke, and the three sources were screenshots on the first
  render.
- Rendering the mock with headless Chromium gave a picture to design against, not a
  stylesheet to imagine.

## 3. What went poorly

- One round of screenshots showed the old renderer because only the app had been
  rebuilt.

## 4. Surprises

- Qt's application font is the one lever that reaches every `Text` item; it has to be
  set before the engine loads, so a face change waits for the next launch.

## 5. Lessons

- PR-rusty-build-workspace-before-shots-001: the screenshot script runs the app and the
  server from `target/debug`; build the workspace, not one crate, before shooting.
- AD-rusty-skin-roles-001: the look is a set of colour roles with three sources (a
  preset, the Omarchy theme, a file); `skin::tokens` derives every token, older names
  included, so a skin is data and QML never learns where it came from.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.5 h |
| 2 Design | 1 h | 1 h |
| 3 Implement | 4 h | 3 h |
| 3.5 Inspect | 0.5 h | 0.5 h |
| 4 Validate | 0.5 h | 0.5 h |
| 5 Complete | 0.5 h | 0.5 h |
