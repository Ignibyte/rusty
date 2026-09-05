---
title: AAR-023-skills-page-layout
ticket: TICKET-023
pipeline: 2e5afa0a-3edb-46f2-aa18-f2c0a1d69379
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-023: Skills page layout

## 0. Recall log

- The splitter TICKET-022 fixed is a file-local `component`; sharing it with a page means
  lifting it into the module, which means a `build.rs` line. That is the one Rust edit.
- Page state goes in `ui` as a JSON string, the way the graph and bookmarks already do.

## 1. Outcome

A draggable split on the Skills page and two collapsible sections, both remembered across
restarts, and one `Splitter.qml` serving the sidebars and the page. Four files. `GATE
GREEN`, two scenes photographed, the state key seen in a scratch state file.

## 2. What went well

- Lifting the splitter instead of copying it: the contract (`value`, `min`, `max`,
  `invert`, `moved`) made the sidebars' call sites *clearer* than the `isLeft` flag was,
  and the page got the 022 fix for free.
- Inspect before the gate again: F1 (a focus ring as a Layout child) would have been a
  runtime warning and a mislaid rectangle; it cost a read instead.

## 3. What went poorly

- Nothing in this one. It was small and the two before it had already paid for the
  pattern.

## 4. Surprises

- The screenshot script's scratch server lists the box's real skills. Read-only in these
  scenes, and TICKET-010's evidence had the same, but a scratch store that is not scratch
  is worth a look at how the script seeds `RUSTY_SKILLS`.
- `Layout.preferredHeight: -1` is the idiom for "no preference" when `fillHeight` takes
  over; a ternary between a cap and `-1` reads oddly but is the smallest correct thing.

## 5. Lessons

- `AD-rusty-one-splitter-owner-clamps-001`: one handle, the owner clamps, the handle
  reports. Page layout is one JSON key on `ui`.
- Before a header becomes interactive, ask what the keyboard does to it (§10). A chevron
  and a `TapHandler` are half of a header.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 20m | 15m |
| 3 Implement | 30m | 20m |
| 3.5 Inspect | 15m | 10m (F1, F2 confirmed) |
| 4 Validate | 20m | 20m |
| 5 Complete | 20m | — |
