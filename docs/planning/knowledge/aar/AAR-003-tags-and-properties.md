---
title: AAR-003-tags-and-properties
pipeline_id: 6d2a9c4e-7b31-4f58-a0e2-9c1d5e7f3b21
ticket: TICKET-003
submitted: 2026-09-03
---

# AAR-003-tags-and-properties

## Recall log

- Register: `AD-rusty-lenient-pages-001`, `AD-rusty-renderer-in-core-001`,
  `PR-rusty-scope-the-sqlite-guard-001`, `PR-rusty-qml-component-scope-001`.
- Completed notes: TICKET-002 (the workspace), TICKET-007 (the wiki at Phase 5).
- Wiki: `vault-and-brain.md`, `workspace-app.md`. Brain: `projects/rusty-v3`.

## 1. Outcomes

- REQ-001 PASS, REQ-002 PASS, REQ-003 PASS, REQ-004 PASS, REQ-005 PASS, REQ-006 PASS.
  Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The property edit sits on the ordered YAML mapping the properties view already used,
  so the editor and the reading view agree on order without a schema change.
- Putting `tag:` inside `brain_search` gave the pane, the rendered tags, the chips and
  any agent one path, and TICKET-005 has a parser to extend.
- The screenshot script showed the editor and the pane on the first build.

## 3. What went poorly

- `serde_yaml::Mapping::remove` is a swap-remove; the first round-trip test caught the
  reordered keys.

## 4. Surprises

- pulldown-cmark's `#tag` rule and the scanner's rule are the same by construction but
  two implementations; both are pinned by tests rather than shared, for now.

## 5. Lessons

- PR-rusty-yaml-mapping-shift-remove-001: removing a key from a `serde_yaml::Mapping`
  is `shift_remove`; `remove` swaps the last key into the hole.
- AD-rusty-tags-one-index-001: frontmatter and inline tags share `brain_tags`, stored
  as first written and compared without case; `tag:` terms are part of `brain_search`,
  not a separate tool.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.5 h |
| 2 Design | 0.5 h | 0.5 h |
| 3 Implement | 2 h | 1.5 h |
| 3.5 Inspect | 0.5 h | 0.5 h |
| 4 Validate | 0.5 h | 0.5 h |
| 5 Complete | 0.5 h | 0.5 h |
