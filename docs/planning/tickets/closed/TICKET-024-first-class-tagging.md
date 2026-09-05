---
title: TICKET-024-first-class-tagging
status: done
ticket_number: 024
type: feature
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/first-class-tagging.spec.md
---

# TICKET-024-first-class-tagging

## Summary

A way to tag a page from the UI: a Tags property type with autocomplete, an add-tag affordance on the note, and tags addable from the Tags pane.

## Why

Tagging works in the back end and is invisible in the front end. `sync_tags()` indexes two sources — the frontmatter `tags:` list and inline `#tags` in the body — and the Tags pane lists every tag with counts and searches on click. But the pane is a read-only browser with no add affordance, and the properties editor's type list is `Text, List, Number, Checkbox, Date` with no Tags type.

So the only ways to tag today are to type `#something` into the body in source view, or to know that a property literally named `tags` of type List will be picked up. Neither is discoverable. Chad asked "how do I tag? Am I missing?" — he was not.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a property is added to a page, Tags shall be offered as a type, and shall write the frontmatter `tags:` list the indexer already reads. | test; smoke |
| REQ-002 | WHEN a tag is being typed into a Tags property, the existing vault tags shall be offered as completions, ranked by count. | smoke |
| REQ-003 | WHEN a page has tags, they shall render as removable pills in the properties block, and removing one shall rewrite the frontmatter. | smoke; screenshot |
| REQ-004 | WHEN a tag in the Tags pane is used to tag the open page, that tag shall be added to the page and the pane's count shall follow. | smoke |
| REQ-005 | WHEN a page is tagged from the UI, `tag:` search, the Tags pane counts and the graph shall reflect it without a restart. | test; smoke |

## Scope

- In: the Tags property type and its editor, autocomplete from the tag index, an add path from the Tags pane, `NoteTab.qml` and `RightPane.qml`.
- Out: renaming a tag across the vault; tag colours; tagging files under a folder root (a file on disk has no frontmatter — that decision belongs with TICKET-019).

## Notes

- Obsidian treats Tags as its own property type; matching that is the point of REQ-001.
- Inline `#tags` keep working and stay the quick path; this ticket is about the deliberate one.
- Pipeline spec: TBC.
