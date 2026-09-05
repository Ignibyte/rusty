---
title: AAR-024-first-class-tagging
ticket: TICKET-024
pipeline: dd540fb5-d193-4833-99fe-842d6f6b55bc
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-024: First-class tagging

## 0. Recall log

- TICKET-005 built the index and the property editor; the front end never said "tag this
  page". Every decision here rides on `AD-rusty-tags-one-index-001`: one index, `tag:`
  in search, a property edit touching the mapping only.
- The window already re-reads `brain_tags` on every change notification, so completions
  and counts are the same read.

## 1. Outcome

Tags as a property type, a completion list under the tags row's field, the Tags pane
tagging the open page, and a palette command that lands the cursor in the field. Three
QML files and one test; `GATE GREEN [diff]`; two scenes photographed twice.

## 2. What went well

- The whole feature rode on TICKET-005's tools: no back-end change beyond a test, and
  the counts, `tag:` search and the graph follow because they always did.
- The offscreen scene found a defect the reading had missed: a `ListView` paints row 0
  as current by default, so the pane looked as if a tag were chosen (F8). A screenshot is
  a test when it is looked at.
- F3 came from imagining one case — `rusty` typed in full beside an existing
  `rust-lang` — and it changed the default from "first pick" to "nothing picked".

## 3. What went poorly

- A `&&` chain ran `cargo build | grep '^Finished'`; cargo indents that line, grep found
  nothing, and the rebuild and the second shoot silently did not run — while the notes
  already said they had. The output was read, the shoot ran after the gate, the notes
  say so. Two rules restated: a step's success is its own exit code, never a grep of
  pretty output; the record is written after the output, not before.

## 4. Surprises

- A delegate's field reached from outside the `Repeater` wants a registration on
  `Component.onCompleted` and a clearing on `onDestruction`, or the reference goes
  stale on the next re-render.
- `Keys.onTabPressed` with `event.accepted = false` hands Tab back to focus navigation
  cleanly; nothing else was needed to keep the field a good citizen.

## 5. Lessons

- `AD-rusty-tag-one-writer-001`: deliberate tagging is one writer on the note, reached
  from the field, the pane and the palette; completions and counts are one read.
- A chained step must not depend on grep over formatted output; test the command's own
  status.
- Look at every scene before recording it; the second look found F8.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 25m | 20m |
