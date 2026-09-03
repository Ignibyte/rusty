---
title: Tags and properties
pipeline_id: 6d2a9c4e-7b31-4f58-a0e2-9c1d5e7f3b21
status: Phase 5 — Complete PASS
ticket: TICKET-003
ticket_doc: docs/planning/tickets/closed/TICKET-003-tags-and-properties.md
aar: docs/planning/knowledge/aar/AAR-003-tags-and-properties.md
sealed: 2026-09-02, Chad: "lets work ticket 2 through 6 auto approved until finished ... I want near identical to obsidian except that we have the shell built in and an MCP in which the agent can interact with"
created: 2026-09-03
---

# Tags and properties: spec

## Intent

Tags and properties the way Obsidian has them: a Tags pane over every tag in the vault,
inline or in frontmatter, with counts and a click that searches; and a properties
block that edits the frontmatter in place with typed values, the body untouched.

## Scope

- In: `brain::links::tags` (the inline scanner), the index (`brain_tags` from both
  sources), `BrainManager::{tags, set_property, remove_property}`, the `tag:` term in
  search, three tools, the Tags pane, the properties editor, the tag links.
- Out (named seams): tag autocomplete, renaming a tag across the vault, other search
  operators (TICKET-005), the graph (TICKET-004).

## Acceptance criteria (EARS)

REQ-001 to REQ-006 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Tags are indexed from both the frontmatter list and the body, into the existing `brain_tags` table, stored as written and compared without case; `#tag` follows Obsidian's rule (a boundary before `#`, letters, digits, `_`, `-`, `/`, at least one letter). | One table, one rule, no schema change. | a second table for inline tags |
| 2 | `tag:` is handled inside `brain_search` (and the hybrid path) as a filter over the tag index, so the pane, the rendered `#tag` links and an agent all use one tool. | No new search tool; TICKET-005 adds the rest of the operators to the same parser. | a separate `brain_pages_by_tag` tool |
| 3 | A property edit rewrites only the frontmatter mapping (order kept, the key set or removed) and writes the body bytes back unchanged; values are JSON (string, number, bool, list of strings), dates are strings shaped `YYYY-MM-DD`. | Obsidian's typed properties map onto YAML scalars and lists; the body is never touched by a property edit. | editing the raw YAML text in place |
| 4 | The properties block in reading view is the editor, as in Obsidian; the source view shows the YAML as text. | One place to edit typed values, one place to edit text. | a separate properties dialog |
| 5 | The Tags pane is a fourth tab of the right sidebar, a tree with counts; a click puts `tag:<name>` into the search pane. | Obsidian's Tags core plugin lives in the right sidebar and searches on click. | a tags section in the explorer |

## Linked artifacts

- Ticket: TICKET-003
- Intake: `docs/planning/intake/INTAKE-knowledge-workspace.md`
- Design references: Chad's Obsidian screenshot (the properties block: title, type,
  created, updated, "Add property"); Obsidian's Tags pane
- Architecture: `openwiki/vault-and-brain.md`, `openwiki/workspace-app.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | sealed by Chad's goal of 2026-09-02 |
| 2 Design | manifest, tool contract, regression table in the notes | design actionable |
| 3 Implement | core, tools, QML | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger | confirmed findings resolved |
| 4 Validate | tests, smoke, screenshots, `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, wiki update, docs, AAR, register, brain capture, archive | pair archived |
