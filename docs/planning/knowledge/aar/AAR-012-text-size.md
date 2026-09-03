---
title: AAR-012-text-size
pipeline_id: c796967f-73d9-4675-9107-40393f31f132
ticket: TICKET-012
submitted: 2026-09-03
---

# AAR-012-text-size

## Recall log

- Register: the workspace-state rule, the QML scope rule, the skin-roles decision.
  TICKET-008 notes on the font. A census of 187 literal sizes across 15 QML files and
  three `pointSize` values; `Style::size` in the renderer.

## 1. Outcomes

- REQ-001 to REQ-006 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- One property read (`theme.scale`) inside every size binding made the whole window
  re-lay out live with no plumbing per file; the sweep was a script over each file's own
  theme reference, 189 sites in one pass.
- The scan test turns the requirement into a gate: a literal size cannot come back
  without failing `cargo test`.

## 3. What went poorly

- The scan test's first rule read only the first character of the value and flagged the
  two editors' derived point sizes; a value that reads `theme.scale` is the right rule.
- Two Settings labels had no wrap and no fill, so at 14 they pushed the column past the
  visible edge and clipped the skin note below them; the larger size surfaced it.

## 4. Surprises

- A `ColumnLayout` given `width: parent.width` still lays out at its children's minimum
  width when one child is wider; the overflow shows on a different child than the one
  that caused it.

## 5. Lessons

- No new register entries. `PR-rusty-workspace-state-in-json-001` and
  `PR-rusty-qml-component-scope-001` held; the wrap-and-fill rule for long labels is
  ordinary QML hygiene.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.3 h | 0.2 h |
| 2 Design | 0.3 h | 0.3 h |
| 3 Implement | 1 h | 0.5 h |
| 3.5 Inspect | 0.2 h | 0.2 h |
| 4 Validate | 0.5 h | 0.4 h |
| 5 Complete | 0.3 h | 0.3 h |
