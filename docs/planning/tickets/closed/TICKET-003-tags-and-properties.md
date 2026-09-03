---
title: TICKET-003-tags-and-properties
status: done
ticket_number: 003
type: feature
created: 2026-09-03
closed: 2026-09-03
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/tags-and-properties.spec.md
---

# TICKET-003-tags-and-properties

## Summary

The second tier of the knowledge workspace: tags, inline and in frontmatter, indexed
and shown in a Tags pane with counts, with a click that searches by tag; and a
properties editor in the page, typed as Obsidian types them, that writes the frontmatter
and leaves the body byte for byte.

## Why

Obsidian users organise by tags and properties as much as by folders; the intake marks
both as build (REQ-006 and REQ-007 there). The workspace shell shows properties read
only and knows only frontmatter tags.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The index shall hold every tag of a page: frontmatter `tags` and inline `#tags` in the body (nested `a/b` allowed; code and URLs skipped), compared without case. | core tests on the scanner and the index |
| REQ-002 | The back end shall list every tag with its page count (`brain_tags`), nested tags counted under their parents too, and `brain_search` shall accept `tag:<name>` terms that restrict results to pages carrying that tag or one nested under it, alone or with words. | core tests; smoke |
| REQ-003 | The right sidebar shall have a Tags pane listing tags with counts as a tree; a click shall search by that tag in the search pane, and a `#tag` in a rendered page shall do the same. | screenshot; keyboard walk by reading |
| REQ-004 | The back end shall set or remove one frontmatter property (`brain_set_property`, `brain_remove_property`) with a JSON value (text, number, checkbox, date, list), keeping the other keys in order and the body byte for byte. | core round-trip tests; smoke |
| REQ-005 | WHEN a page is in reading view, its properties block shall be editable in place: a value edits by type (text and date fields, number, a checkbox, list chips added and removed), a row can be removed, and "Add property" adds a key with a chosen type. | screenshot; smoke of the tools |
| REQ-006 | The status bar's property count and the reading view shall follow a property edit without a reload of the tab. | screenshot after an edit through the tool; reading |

## Scope

- In: the scanner and index for inline tags, the tags tool and the `tag:` term, the
  Tags pane, the properties tools and editor.
- Out: tag autocomplete while typing (later), a tags pane that renames tags across the
  vault (later), search operators beyond `tag:` (TICKET-005), the graph (TICKET-004).

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/tags-and-properties.spec.md`
- Related docs: `openwiki/vault-and-brain.md`, `openwiki/workspace-app.md`
- Promoted from intake: `INTAKE-knowledge-workspace` (REQ-006, REQ-007 there)
- Follow-ups opened: tag autocomplete while typing and renaming a tag across the vault stay with the later tiers
