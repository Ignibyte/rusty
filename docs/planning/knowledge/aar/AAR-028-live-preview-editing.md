---
title: AAR-028-live-preview-editing
ticket: TICKET-028
pipeline: 3c7a7dd7-ed6c-4903-8af3-75505c3b5475
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-028: Live preview editing

## 0. Recall log

- The reading view already renders one block per section; the grain of live preview
  was decided by that, not by Obsidian's span-level editor, which is a seam.
- The tokenizer's path to C++ (a bridge function, a thin subclass) is the pattern the
  split follows to reach QML.

## 1. Outcome

Three views: reading, live preview and source, live preview the default editing mode
and the user's choice kept. Live preview at the section grain: a click on a rendered
block opens that section alone as a highlighted source editor, the rest stays rendered,
the same autosave and `Ctrl+S` save the page, no re-render while a section is open. One
Rust function with a test, one C++ invokable, the note tab and one state key. `GATE
GREEN [diff]`; two scenes.

## 2. What went well

- The grain decision made the largest workspace ticket small: the reading view already
  rendered one block per section, so live preview became "open this block's source",
  and the fallback to the whole-file editor covers what the split cannot line up.
- `page_sections`' identity assertion — the parts concatenate to the page — is the
  whole safety net for a save that assembles the file from parts.
- The tokenizer's path to C++ was the pattern; `Tools.pageSections` is twelve lines.

## 3. What went poorly

- `tools: tools` on the note instance bound the tab's own property to itself — the
  wiki's own invariant names this — and the first scene showed the fallback instead of
  a section. Named apart (`sectionTools`), and the scene showed the section.

## 4. Surprises

- The fallback made the defect visible: a wrong split would have edited the whole body,
  never the wrong section. A design that fails loudly toward the safe mode is worth
  the extra branch.

## 5. Lessons

- `AD-rusty-live-preview-is-a-section-001`: the construct is a section; the split is
  Rust's and joins back byte for byte; a mismatch edits the whole file.
- An inline component's property shadows an id of the same name: bind through the
  window or name the property apart.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 30m | 25m |
