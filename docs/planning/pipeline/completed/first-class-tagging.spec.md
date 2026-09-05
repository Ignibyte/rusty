---
title: First-class tagging
pipeline_id: dd540fb5-d193-4833-99fe-842d6f6b55bc
status: Phase 5 — Complete PASS
ticket: TICKET-024
ticket_doc: docs/planning/tickets/open/TICKET-024-first-class-tagging.md
aar: docs/planning/knowledge/aar/AAR-024-first-class-tagging.md
sealed: no new tab, tool, table or dependency; the tag index and the property tools of TICKET-005 carry it
created: 2026-09-05
---

# First-class tagging: spec

## Intent

Tagging works in the back end and is invisible in the front end: the index reads the
frontmatter `tags:` list and inline `#tags`, the Tags pane lists and searches, the
properties block edits a list — and nothing says "tag this page". Chad asked how to tag.
This ticket is the deliberate path: a Tags type when a property is added, completions
from the vault's tags while typing, the open page tagged from the Tags pane, and a
palette command that lands the cursor in the tags field.

## Scope

- In: the Tags property type; a completion list under the tags row's field; a
  `tagThePage` function on the note that the pane and the palette call; the Tags pane's
  `+`, right-click menu and `T` key; the palette command; a `tagfield:` scene; the core
  test extended for the counts and `tag:` search after a property write.
- Out (named seams, not forgotten): renaming a tag across the vault; tag colours; tagging
  a file under a folder root (no frontmatter; TICKET-019 left it out); completions for
  inline `#tags` in the source editor.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a property is added to a page, Tags shall be offered as a type, and shall write the frontmatter `tags:` list the indexer already reads. | core test (`set_property("tags", […])` → the file, the counts); reading of `addProperty`; the `reading` scene |
| REQ-002 | WHEN a tag is being typed into a Tags property, the existing vault tags shall be offered as completions, ranked by count. | offscreen scene `tagfield:r`; reading of `tagCompletions` |
| REQ-003 | WHEN a page has tags, they shall render as removable pills in the properties block, and removing one shall rewrite the frontmatter. | the `reading` scene (the chips, each with ×); reading of the chip's remove path (TICKET-005, unchanged) |
| REQ-004 | WHEN a tag in the Tags pane is used to tag the open page, that tag shall be added to the page and the pane's count shall follow. | reading of `tagPage` → `tagThePage` and of the `brain_tags` re-read on the change notification; smoke by Chad |
| REQ-005 | WHEN a page is tagged from the UI, `tag:` search, the Tags pane counts and the graph shall reflect it without a restart. | core test: after `set_property("tags", …)` the counts and `tag:` search follow, and follow again when a tag is removed |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Tags is a type in the add-property list whose key is fixed to `tags` and whose value is a list; a page that already has `tags` gets its row's field focused rather than a second key or an overwrite | The indexer reads `tags:` already (TICKET-005 decision 1); Obsidian's Tags type is the same key; nothing new in the file format | a separate `#tags` property; a tags field outside the properties block |
| 2 | Completions come from the tag index the window already holds (`win.tags`, re-read on every change notification), filtered in QML: a substring match without case, tags already on the page left out, ranked by count then name, eight shown | No new tool; the pane's list and the completions are one read; ranking by count is what REQ-002 asks | a `brain_tag_complete` tool; prefix-only matching |
| 3 | One function writes: `NoteTab.tagThePage(tag)` adds a tag to the list (creating `tags` when absent, refusing a duplicate without case); the pane's `+`, its menu, its `T` key and the palette all call it through the window | One writer, one rule for duplicates; the pane never writes on its own | the pane calling `brain_set_property` itself |
| 4 | The keyboard path is a palette command, "Tags: Tag this page", that focuses the tags row's field (adding the property first when the page has none); Enter or Tab takes the highlighted completion, Enter with none adds the text as typed | Keyboard first (§10); the completion list is reachable without a mouse | a chord (the free ones are few and this is not a daily key) |
| 5 | The Tags pane's rows become keyboard rows (`currentIndex`, Enter searches, `T` tags); the pane's `+` shows on hover and the right-click menu carries both actions | The pane was mouse-only; two actions on one row want a menu | a text field at the top of the pane |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-024-first-class-tagging.md`
- Tags and properties: `docs/planning/pipeline/completed/tags-and-properties.spec.md`,
  `AD-rusty-tags-one-index-001`
- Architecture: `openwiki/workspace-app.md` (the properties block, the Tags pane),
  `openwiki/vault-and-brain.md` (the tag index)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; no seal needed |
| 2 Design | Manifest, the completion list, the writer, the pane, regression table | design actionable |
| 3 Implement | `NoteTab.qml`, `RightPane.qml`, `Main.qml`, the core test | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over the property path | confirmed findings resolved |
| 4 Validate | The test, two offscreen scenes, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
