---
title: Knowledge workspace shell: notes
pipeline_id: 6b0c5d3e-2c0f-4b7e-9f0a-1d2e3f4a5b6c
---

# Knowledge workspace shell: running notes

## Phase 1: Plan

- Recall: the app already has a Brain tab (type tree, search, page view with rendered
  markdown and clickable wikilinks, edit and save of the compiled truth, timeline append,
  links, open in Obsidian, capture) and a Notes tab (daily pages). The core has the vault
  manager (one folder level, the type table), frontmatter parsing with the `## Timeline`
  section, an FTS and link index, the semantic index, page versions, and the Obsidian
  bridge. Qt's `Text.MarkdownText` is what renders today; it has no callouts, embeds or
  footnotes, and QtWebEngine is installed on the box but is not the path (decision 1).
  Chad's three screenshots (the Tasks tab, Obsidian's graph, an Obsidian page) and the
  Replit mock are the visual references; the mock is in `docs/design/`.
- Decisions: the six in the spec.
- Seal: pending Chad.

## Phase 2: Design

- Architecture and data flow:
- File manifest:
- Store consequences:
- Tool contract:
- Regression plan:
- Risks:
- CodeGraph evidence:

## Phase 3: Implement

- Built:
- Deviations:
- Fast gate:

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|

- Post-implementation CodeGraph:

## Phase 4: Validate

- Tests run (commands and output):
- Gate run:
- Smoke evidence:
- Skips or pre-existing failures:

## Phase 5: Complete

- Requirement audit:
- Docs:
- AAR:
- Brain capture:
- Archive:

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
