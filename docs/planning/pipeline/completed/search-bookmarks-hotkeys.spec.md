---
title: Search operators, bookmarks and hotkeys
pipeline_id: 833ac963-574a-478e-9790-ae31bcaccb8c
status: Phase 5 — Complete PASS
ticket: TICKET-005
ticket_doc: docs/planning/tickets/closed/TICKET-005-search-bookmarks-hotkeys.md
aar: docs/planning/knowledge/aar/AAR-005-search-bookmarks-hotkeys.md
sealed: 2026-09-02, Chad: "lets work ticket 2 through 6 auto approved until finished ... I want near identical to obsidian except that we have the shell built in and an MCP in which the agent can interact with"
created: 2026-09-03
---

# Search operators, bookmarks and hotkeys: spec

## Intent

Search that narrows the way Obsidian's does (operators, match case, regex), bookmarks in
the sidebar tab that already waits for them, and a table of the keys.

## Scope

- In: a query parser in core shared by full-text, hybrid, the tool and the CLI; match
  case and regex modes over the indexed text; `brain_search` parameters; the search
  pane's toggles, hint and bookmark action; `BookmarksPane.qml` and the `bookmarks` key of
  the workspace state; entry points in the page menu, the explorer, the outline and the
  palette; the Hotkeys table in Settings; screenshot scenes.
- Out (named seams): search and replace, `line:` and `section:`, bookmark groups and
  ordering, custom keys, a bookmarks tool.

## Acceptance criteria (EARS)

REQ-001 to REQ-006 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Operators parse once in core (`parse_query`), for full-text, hybrid, the tool and the CLI alike: `tag:` through `brain_tags`, `path:`, `file:` and `type:` against the indexed page rows, `-` excluding. | One parser, one meaning of a query everywhere agents and the app search. | parsing in QML; separate tool parameters per operator |
| 2 | Match case and regex scan the indexed text in Rust (the `regex` crate), with the operators applied first; regex drops the vector half of a hybrid search. | FTS5 has neither; the index already holds every page's text; vectors have no meaning for a pattern. | a second FTS table; scanning files |
| 3 | Bookmarks live in the workspace state file under `bookmarks` (kind, target, title), as Obsidian keeps them in the vault's own config; no tool yet. | They are the user's view of the vault, not vault content; the state file already round-trips. | a store table and a tool |
| 4 | The Hotkeys table is the palette's command list rendered read-only, with the terminal keys added; assignment is a later ticket. | Defaults first, as the intake says; one list feeds the palette and the table. | a keymap file now |

## Linked artifacts

- Ticket: TICKET-005
- Intake: `docs/planning/intake/INTAKE-knowledge-workspace.md`
- Design references: Obsidian's search pane (the `Aa` and `.*` toggles beside the field,
  operators in the query), its Bookmarks tab, its Hotkeys settings tab
- Architecture: `openwiki/workspace-app.md`, `openwiki/vault-and-brain.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | sealed by Chad's goal of 2026-09-02 |
| 2 Design | manifest, tool contract, regression table in the notes | design actionable |
| 3 Implement | core, tool, QML | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger | confirmed findings resolved |
| 4 Validate | tests, smoke, screenshots, `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, wiki update, docs, AAR, register, brain capture, archive | pair archived |
