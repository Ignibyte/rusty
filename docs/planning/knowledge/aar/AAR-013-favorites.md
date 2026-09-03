---
title: AAR-013-favorites
pipeline_id: 44e4d03a-c884-487c-a38f-f8120dd6bb1a
ticket: TICKET-013
submitted: 2026-09-03
---

# AAR-013-favorites

## Recall log

- Register: the bookmarks decision and the QML scope rule. TICKET-005 notes. Code: the
  window's bookmark functions, the note's `bookmarked` property, the explorer's rows,
  the switcher's scoring. Ctrl+D unbound.

## 1. Outcomes

- REQ-001 to REQ-005 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- TICKET-005 had built the storage, the toggle and the open paths; the whole ticket was
  four affordances over functions that existed, and the scratch state already held a
  bookmarked file and folder, so the scenes proved it without new fixtures.

## 3. What went poorly

- The ticket asked for a "pure function" test on the switcher's ordering; the app has no
  QML unit harness, so the design replaced it with a scene against a known state. Worth
  a line in the ticket template's verification column next time: name the harness that
  exists.

## 4. Surprises

- None.

## 5. Lessons

- No new register entries; `AD-rusty-bookmarks-in-state-001` held: the user's view of the
  vault lives in the workspace state, and a second surface over it costs nothing in the
  store.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.3 h | 0.2 h |
| 2 Design | 0.3 h | 0.2 h |
| 3 Implement | 0.5 h | 0.3 h |
| 3.5 Inspect | 0.2 h | 0.1 h |
| 4 Validate | 0.3 h | 0.3 h |
| 5 Complete | 0.3 h | 0.3 h |
