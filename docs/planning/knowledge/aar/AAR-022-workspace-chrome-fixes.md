---
title: AAR-022-workspace-chrome-fixes
ticket: TICKET-022
pipeline: e8f444c2-71de-4e14-8e1a-fff6b5c7ca14
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-022: Workspace chrome fixes

## 0. Recall log

- Four bugs, four causes found by reading before writing: a moving coordinate frame, a
  documented feature that was never built, a dialog child with no layout, and a plus
  wired to one thing. The register had nothing on any of them; `TasksPage.qml` had the
  drag pattern to copy.

## 1. Outcome

All four fixed in two QML files: the splitters measure in scene coordinates; tabs drag
to reorder through the existing `moveTab`; the rename dialog has a layout; the `+` is a
menu of the switcher, every top-bar agent, and the custom terminal. The Settings sentence
that promised tab drag is now true and mentions the `+`. `GATE GREEN [diff]`; both new
surfaces photographed offscreen.

## 2. What went well

- Inspect ran before the fast gate this time and paid twice: F8 (the tooltip lost its
  keyboard answer) and F9 (a wrong comment) were caught reading, not building.
- Copying the Tasks page's `DragHandler` rather than inventing a drag: the same shape,
  one axis flipped, worked first time.
- Adding two `RUSTY_SHOT_SCENE` values (`rename`, `plus`) made the dialog and the menu
  photographable offscreen; the first shot of the menu is what found F14.

## 3. What went poorly

- F14 — the truncated agent labels — was only visible in the screenshot. The reading
  passes assumed the palette's phrasing would fit a menu; it did not. Evidence beats
  reasoning about layout.

## 4. Surprises

- `cargo build` prints "the gold linker is deprecated and has known bugs with Rust". It
  is the box's linker configuration, invisible to clippy, and predates this work. Worth
  a line in the dev-box handbook rather than a fix here.
- A `Repeater` cannot populate a `Menu`; `Instantiator` with `insertItem` can, and the
  explicit index makes the placement between two static separators deterministic.

## 5. Lessons

- `BF-rusty-moving-frame-delta-001`: never measure a drag in the frame of the thing the
  drag moves. Scene coordinates, or a `DragHandler`.
- Photograph new chrome before calling it done; the menu's labels were fine in prose.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 15m |
| 2 Design | 20m | 15m |
| 3 Implement | 30m | 20m |
| 3.5 Inspect | 15m | 15m (F8, F9 confirmed; F14 later from evidence) |
| 4 Validate | 20m | 30m (two gates, two builds, three shots) |
| 5 Complete | 20m | — |
