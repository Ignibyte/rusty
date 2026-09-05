---
title: Live preview editing
pipeline_id: 3c7a7dd7-ed6c-4903-8af3-75505c3b5475
status: Phase 5 — Complete PASS
ticket: TICKET-028
ticket_doc: docs/planning/tickets/open/TICKET-028-live-preview-editing.md
aar: docs/planning/knowledge/aar/AAR-028-live-preview-editing.md
sealed: no new tab, tool, table or dependency; one Rust function, one C++ invokable, QML in the page tab; the file format untouched
created: 2026-09-05
---

# Live preview editing: spec

## Intent

A page opens in reading view and `Ctrl+E` flips it to the whole-file source editor;
the mode people live in — click into rendered text and type, with only the construct
under the caret showing its markup — is missing. This pipeline adds it at the grain
the reading view already has: a section (a heading and what follows it) is the
construct. In live preview a click on a section turns that section alone into a
highlighted source editor; every other section stays rendered; the same autosave and
`Ctrl+S` save the page; the editing mode is the user's and persists.

## Scope

- In: a third mode, live preview, as the default editing mode; the section split in
  Rust (`page_sections`: the frontmatter, then one part per heading outside fenced
  code, joined byte for byte), reached from QML through `Tools`; the page tab's live
  mode (click to edit a section, the caret near the clicked line, Escape or a click
  elsewhere to commit, re-render held while a section is being edited); the editing
  mode under `editMode` in the workspace state with three palette commands and the
  header's `[ READ ]`/`[ LIVE ]`/`[ EDIT ]`; a `live:<n>` scene.
- Out (named seams, not forgotten): the construct-under-caret grain of Obsidian's
  Live Preview (a span rather than a section: the renderer would have to map rendered
  positions to source offsets); WYSIWYG toolbars; tables as grids; any change to the
  file format; a caret placed at the exact character clicked (it lands on the line
  the click's height points at).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a page is open in live preview and text is clicked, the caret shall land at that point and typing shall edit the page. | reading of `editSection` (the section's editor takes focus, the caret on the line the click's height points at); the `live:1` scene; smoke by Chad |
| REQ-002 | WHEN the caret is inside a markdown construct, that construct shall show its source; every other construct shall stay rendered. | `page_sections_split_at_headings_outside_fences`; the scene (one section as source with the highlighter, the rest rendered) |
| REQ-003 | WHEN a page is edited in live preview, it shall autosave on the same schedule and on `Ctrl+S` as the source editor does. | reading: the section editor restarts the same `autosave` timer and its `Ctrl+S` calls the same `save`, which assembles the page from the parts and the live section |
| REQ-004 | WHEN the view is switched, reading, live preview and source shall each be reachable, and the choice shall persist per the user, not per page. | reading of `ui.editMode` (load, save, write) and the three commands; smoke across a restart by Chad |
| REQ-005 | WHEN a page is in live preview, wikilinks, tags, embeds and task checkboxes shall stay clickable. | reading: the rendered blocks are the same `Text` items with `onLinkActivated`, and a tap on a link never enters a section; the scene |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The construct is a section: the text from one heading (any level, outside fenced code) to the next, the preamble before the first heading its own section; the reading view already renders one block per section (`HEADING_MARK`), so the split in `page_sections` mirrors the renderer's rule and the blocks line up one to one | The grain the app already has; a span-level live preview needs a position map the renderer does not keep, and is a seam | a `QSyntaxHighlighter` that hides markup (the markup still takes width); a full editor component |
| 2 | The split lives in Rust (`markdown::page_sections`, the frontmatter as element 0, `join` giving the file back byte for byte) and reaches QML through a `Tools` invokable, as the tokenizer reaches the highlighter through C++ | One rule, tested without Qt; the C++ side stays thin | the split in QML (untested); a QML type of its own |
| 3 | When the parts and the rendered blocks do not line up (a setext heading, a heading the renderer treats otherwise), live preview edits the whole body in the source editor rather than guess | Never edit the wrong text; the fallback is the mode that always works | trusting the count |
| 4 | While a section is being edited the page is not re-rendered: a save keeps the blocks as they are and a change notification waits; committing the section (Escape, a click elsewhere, leaving edit mode) renders again | A re-render rebuilds the blocks and would take the caret and the text out from under the writer | re-rendering on every save |
| 5 | The editing mode (`live` or `source`) is `ui.editMode`, default `live`; `Ctrl+E` and the header toggle reading against that mode; "View: Reading view", "View: Live preview" and "View: Source mode" set it directly | Per the user, not per page; the default is the mode the ticket is for | a per-page mode; a fourth toggle state |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-028-live-preview-editing.md`
- Register: `AD-rusty-renderer-in-core-001` (the reading view is the renderer's HTML),
  `AD-rusty-workspace-is-obsidian-001`
- Architecture: `openwiki/workspace-app.md` (the page tab)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled |
| 2 Design | Manifest, the split, the section editor, the hold on re-render, regression table | design actionable |
| 3 Implement | `markdown.rs`, `cpp/tools.*`, `NoteTab.qml`, `Main.qml` | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over the split | confirmed findings resolved |
| 4 Validate | The test, the scene, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
