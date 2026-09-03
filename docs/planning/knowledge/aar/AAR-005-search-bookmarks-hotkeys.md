---
title: AAR-005-search-bookmarks-hotkeys
pipeline_id: 833ac963-574a-478e-9790-ae31bcaccb8c
ticket: TICKET-005
submitted: 2026-09-03
---

# AAR-005-search-bookmarks-hotkeys

## Recall log

- Register: `AD-rusty-tags-one-index-001`, `AD-rusty-workspace-is-obsidian-001`,
  `PR-rusty-workspace-state-in-json-001`, `PR-rusty-qml-signal-names-001`.
- Completed notes: TICKET-003 (tag search), TICKET-004 (state keys, scenes).
- Wiki: `workspace-app.md`, `vault-and-brain.md`. Brain: `projects/rusty-v3`.

## 1. Outcomes

- REQ-001 PASS, REQ-002 PASS, REQ-003 PASS, REQ-004 PASS, REQ-005 PASS, REQ-006 PASS.
  Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The tag search from TICKET-003 already had the shape (words, an allowed set,
  `pages_in`); the operators slotted into it and the old signatures stayed as wrappers,
  so the CLI, the tool and the tests needed no change to keep working.
- The screenshot script's seeded state made the Bookmarks pane's round trip a scene
  rather than a hand check.

## 3. What went poorly

- One compile cycle went to a block whose tail expression borrowed the statement the
  block owned.

## 4. Surprises

- None.

## 5. Lessons

- PR-rusty-collect-inside-scope-001: when a block owns a connection guard or a prepared
  statement, collect the rows into a named `Vec` inside it; a tail expression that
  still borrows them does not compile.
- AD-rusty-search-operators-in-core-001: one parser (`parse_query`) gives a query the
  same meaning in the pane, the tool, the CLI and hybrid search; the text modes are
  scans over the indexed text, not a second index.
- AD-rusty-bookmarks-in-state-001: bookmarks are the user's view of the vault, kept
  under `bookmarks` in the workspace state like the sidebars, not vault content.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.25 h |
| 2 Design | 0.5 h | 0.5 h |
| 3 Implement | 2 h | 1.5 h |
| 3.5 Inspect | 0.5 h | 0.25 h |
| 4 Validate | 0.5 h | 0.5 h |
| 5 Complete | 0.5 h | 0.5 h |
