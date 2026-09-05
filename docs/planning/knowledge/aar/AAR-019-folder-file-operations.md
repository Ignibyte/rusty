---
title: AAR-019-folder-file-operations
ticket: TICKET-019
pipeline: 1140d5ef-f95d-4c95-a05e-8e977e5af3f5
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-019: Folders, part two — file operations

## 0. Recall log

- Part one's decisions carried straight into part two: writes through the same Rust type,
  disk rows in the one list, the listing cache dropped on every write.
- `chrono` and `dirs` were already dependencies of `rusty-app`, so the trash record's date
  and the XDG data directory cost nothing new — worth checking before assuming, which an
  earlier draft of this line did not.

## 1. Outcome

Six disk writes behind the explorer's menu, its keys and a drag; an edit mode in the file
tab; the trash in place of a delete. Four files, five tests. `GATE GREEN [diff]` twice
(F16 came after the first), two scenes photographed.

## 2. What went well

- The shape from part one held: pure functions with a `Result`, one `outcome()` for the
  JSON, thin invokables. The tests are on the functions and the bridge stays one line each.
- The first test run caught F1 (a file dropped on a file answered "cannot move into
  itself"): the order of two guards decides which message a user reads, and only a test
  with both cases says which order is right.
- CodeGraph's blast radius after the change matched the manifest exactly; nothing outside
  `folders.rs` and its two QML callers moved.

## 3. What went poorly

- The QML patch script ran with repo-relative paths from a Bash session whose working
  directory had drifted into `crates/rusty-app/qml`; it failed before writing anything and
  a turn was spent finding out. Absolute paths, or `cd` to the root first, every time.
- F16 turned up at complete, not at inspect: part one already had a `file:` scene taking
  an absolute path, and the new home-relative branch sat above it and shadowed it. A grep
  for the prefix before adding a scene branch would have found it in a second.

## 4. Surprises

- `scripts/screenshot.sh` seeds no plain file under its scratch `HOME`, so one of its own
  vault pages stood in as the disk file for the edit scene. It served; a seeded text file
  would read more like the feature.
- Reload while dirty needed a decision: refuse with a notice rather than discard or save.
  The notice names Ctrl+S, so the refusal is also the instruction.

## 5. Lessons

- `AD-rusty-disk-writes-refuse-and-trash-001`: the app's disk writes refuse what exists,
  trash rather than delete, and rename rather than overwrite; the back end never sees a
  root.
- When two guards can both fire on one input, write the test for that input and let it
  pick the order (F1).
- Before adding a scene, environment switch or command prefix, grep for the prefix; an
  older branch may already own it (F16).

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 20m | 15m |
| 2 Design | 30m | 25m |
