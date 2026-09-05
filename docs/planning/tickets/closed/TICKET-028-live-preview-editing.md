---
title: TICKET-028-live-preview-editing
status: done
ticket_number: 028
type: feature
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/live-preview-editing.spec.md
---

# TICKET-028-live-preview-editing

## Summary

Edit a page in place in the rendered view, the way Obsidian's Live Preview does, instead of toggling to raw source.

## Why

A page opens in reading view (`NoteTab`'s `editing` starts false) and `Ctrl+E` or the `[ READ ]` button flips it to the **source** editor. So Rusty has two of Obsidian's three views — Reading and Source — and is missing the one people actually live in, where you click into rendered text and type, with only the construct under the cursor showing its markup.

Chad's report was "when you have a file open you cant edit it unless you click read", which is the toggle being felt as friction. The toggle is not the problem; the missing third mode is.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a page is open in live preview and text is clicked, the caret shall land at that point and typing shall edit the page. | smoke |
| REQ-002 | WHEN the caret is inside a markdown construct, that construct shall show its source; every other construct shall stay rendered. | smoke; screenshot |
| REQ-003 | WHEN a page is edited in live preview, it shall autosave on the same schedule and on `Ctrl+S` as the source editor does. | test; smoke |
| REQ-004 | WHEN the view is switched, reading, live preview and source shall each be reachable, and the choice shall persist per the user, not per page. | smoke across a restart |
| REQ-005 | WHEN a page is in live preview, wikilinks, tags, embeds and task checkboxes shall stay clickable. | smoke |

## Scope

- In: a third view mode in `NoteTab.qml` and whatever the back end's renderer needs to support caret-aware rendering.
- Out: WYSIWYG toolbars; tables edited as grids; changing the file format in any way.

## Notes

- This is the largest of the workspace tickets and should not be bundled with the small fixes in TICKET-022.
- Two cheaper halves exist if it needs staging: a setting for which view a page opens in, and click-to-edit that drops into the source editor at the clicked line.
- Pipeline spec: TBC.
