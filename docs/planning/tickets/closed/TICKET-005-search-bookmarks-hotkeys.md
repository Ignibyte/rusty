---
title: TICKET-005-search-bookmarks-hotkeys
status: done
ticket_number: 005
type: feature
created: 2026-09-03
closed: 2026-09-03
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/search-bookmarks-hotkeys.spec.md
---

# TICKET-005-search-bookmarks-hotkeys

## Summary

The last of the workspace tiers before the bridge goes: Obsidian's search operators and
its match-case and regex toggles, bookmarks of files, folders, searches and headings in
the left sidebar, and a hotkeys table in Settings.

## Why

Search is how a vault of a few hundred pages is used day to day; Obsidian's operators
(`path:`, `file:`, `tag:`, `type:` here) and the two toggles are what narrow a search.
Bookmarks are the third tab of Obsidian's left sidebar and the app shows it dimmed.
The hotkeys table is where a user learns the keys; the intake marks it build, defaults
first.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The search shall accept `path:`, `file:`, `tag:` and `type:` terms (a value in quotes may hold spaces; a leading `-` excludes) together with words, and a query of operator terms alone shall list the matching pages newest first. | core tests; smoke |
| REQ-002 | WHEN match case or regex is on, the search shall match the page text that way and return a snippet around the first match, with the operators still applied. | core tests |
| REQ-003 | The search pane shall carry the match-case and regex toggles, a way to bookmark the search, and a hint of the operators while the field is empty. | screenshot |
| REQ-004 | The app shall keep bookmarks of files, folders, searches and headings in a Bookmarks pane of the left sidebar, added from the page menu, the explorer, the search pane, the outline and the palette, remembered between runs; a click opens the target and a menu removes or retitles the bookmark. | screenshot; state round trip by reading |
| REQ-005 | Settings shall show a Hotkeys table of every command with its key, with a filter field. | screenshot |
| REQ-006 | `brain_search` shall accept `case_sensitive` and `regex` as optional parameters, and its description shall name the operators; existing calls keep working. | router test; smoke |

## Scope

- In: the query parser and the two modes in core, the tool parameters, the search pane
  toggles and hint, the Bookmarks pane and its entry points, the Hotkeys table, the
  screenshot scenes.
- Out: search and replace across the vault, `line:` and `section:` operators, bookmark
  groups and drag ordering, custom key assignment, a bookmarks tool for agents (a seam if
  agents need them).

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/search-bookmarks-hotkeys.spec.md`
- Related docs: `openwiki/workspace-app.md`, `openwiki/vault-and-brain.md`,
  `openwiki/mcp-back-end.md`
- Promoted from intake: `INTAKE-knowledge-workspace` (the Search table and the
  bookmarks and hotkeys rows)
- Follow-ups opened: a bookmarks tool for agents if they need one; custom key assignment
