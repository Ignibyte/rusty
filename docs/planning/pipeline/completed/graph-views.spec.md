---
title: Graph views
pipeline_id: 9e4b7f12-3c8a-4d6e-b1f5-7a2c9d0e4f63
status: Phase 5 — Complete PASS
ticket: TICKET-004
ticket_doc: docs/planning/tickets/closed/TICKET-004-graph-views.md
aar: docs/planning/knowledge/aar/AAR-004-graph-views.md
sealed: 2026-09-02, Chad: "lets work ticket 2 through 6 auto approved until finished ... I want near identical to obsidian except that we have the shell built in and an MCP in which the agent can interact with"
created: 2026-09-03
---

# Graph views: spec

## Intent

The vault as Obsidian's graph shows it: pages as dots sized by their links, links as
lines, laid out by forces, with the filters, groups, display and force settings in a
foldable panel; and the same around one page as a local graph.

## Scope

- In: `BrainManager::graph` and `brain_graph`, `GraphView.qml` (canvas, forces, panel,
  interaction), the `graph` tab kind (global and local), the ribbon button, Ctrl+G, the
  palette commands, "Open local graph" in the page menu, the settings in the workspace
  state, a screenshot scene.
- Out (named seams): attachment nodes, group presets, export.

## Acceptance criteria (EARS)

REQ-001 to REQ-006 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The graph data comes from the index through one tool; the layout runs in the app (a force simulation in QML JavaScript on a `Canvas`), because layout is view state: it moves with the user's drags and settings. | A few hundred nodes lay out in real time in QML; the tool stays small and serves agents too. | layout in Rust behind a QObject; a web view |
| 2 | Node radius grows with the square root of the degree; labels fade in with zoom past a threshold and always show on hover, as Obsidian's do. | Obsidian's look. | labels always on |
| 3 | Groups are queries (`tag:`, `path:`, `type:` or plain text against title and slug) with a colour chosen from the theme's palette, first match wins. | Obsidian's groups are queries with colours; the theme keeps the palette. | a free colour picker |
| 4 | The local graph is the same view with `around` and `depth` set, opened from the page menu or the palette; it follows the open page while its tab is showing. | One view, one code path. | a separate component |
| 5 | Graph settings live in the workspace state file (`graph` key) so they survive restarts. | The same place the sidebars live. | per-vault settings in the store |

## Linked artifacts

- Ticket: TICKET-004
- Intake: `docs/planning/intake/INTAKE-knowledge-workspace.md`
- Design references: Chad's Obsidian graph screenshot (Filters, Groups, Display, Forces
  panel at the top right; dots in the accent colour; faint lines)
- Architecture: `openwiki/workspace-app.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | sealed by Chad's goal of 2026-09-02 |
| 2 Design | manifest, tool contract, regression table in the notes | design actionable |
| 3 Implement | core, tool, QML | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger | confirmed findings resolved |
| 4 Validate | tests, smoke, screenshots, `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, wiki update, docs, AAR, register, brain capture, archive | pair archived |
