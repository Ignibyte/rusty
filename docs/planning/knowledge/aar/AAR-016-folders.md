---
title: AAR-016-folders
pipeline_id: bb302c74-c180-4ce4-aea4-68a6ff889539
ticket: TICKET-016
submitted: 2026-09-03
---

# AAR-016-folders

## Recall log

- Register and code as in the notes; the seal relayed at 17:20 with the rustal
  session's recommendations as the answers.

## 1. Outcomes

- REQ-001, REQ-002, REQ-003 and REQ-006 PASS; REQ-004 and REQ-005 minted as
  TICKET-019 and TICKET-020. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The disk rows joined the explorer's one list with their own kinds, so keys, the
  current row and scrolling needed no second component.
- The scene runner took `root:` and `file:` parts, so the evidence needed no script change.

## 3. What went poorly

- The `folders: folders` binding on the file tab and the explorer named the required
  property itself (the third time this scope rule bit: `PR-rusty-qml-component-scope-001`).
- A `rm -f $S/*` in a screenshot step raised a permission prompt that stalled the session
  for 28 minutes until Chad pressed Yes (the rustal session noticed).

## 4. Surprises

- An empty `renaming` string matched the section row's empty path, so the rename field
  appeared on it; the `expanded` map, keyed by path, took absolute paths without a clash.

## 5. Lessons

- Give a QML instance an id that no consumer's property shares; bind `folders:
  diskFolders`, never `folders: folders`.
- Clear a scratch folder with `find "$S" -maxdepth 1 -type f -delete` or use a new folder
  per shot; a glob under an unquoted variable prompts and stalls.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15 min | 10 min |
| 2 Design | 20 min | 20 min |
| 3 Implement | 90 min | 60 min (plus the 28-minute stall) |
| 3.5 Inspect | 10 min | 10 min |
| 4 Validate | 15 min | 10 min |
| 5 Complete | 30 min | 30 min |
