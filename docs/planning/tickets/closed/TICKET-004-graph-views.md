---
title: TICKET-004-graph-views
status: done
ticket_number: 004
type: feature
created: 2026-09-03
closed: 2026-09-03
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/graph-views.spec.md
---

# TICKET-004-graph-views

## Summary

Obsidian's graph views inside Rusty: a global graph of pages and their links, laid out by
forces, with filters, colour groups, display settings and force settings in a panel at
the side; and a local graph around the open page with a depth. A node click opens the
page.

## Why

The graph is how Obsidian shows the shape of a vault (Chad's second screenshot is his
own vault's graph), and the intake marks it as build (REQ-008 there). The index already
holds every link and tag; the app lacks the view.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The back end shall return the vault's graph (`brain_graph`): page nodes with title, type and folder, and edges from resolved links; on request tag nodes with their edges, unresolved targets as nodes, and a neighbourhood around one page to a depth. | core tests; smoke |
| REQ-002 | The app shall open a Graph view as a tab (ribbon, Ctrl+G, the palette) that lays the nodes out by forces (centre, repel, link, link distance) and draws them on a canvas: nodes sized by degree, links as lines, labels that appear as the view zooms in or on hover. | screenshots; keyboard walk by reading |
| REQ-003 | The graph shall pan with a drag on the background and zoom with the wheel; a node shall drag, highlight itself and its neighbours on hover, show its title, and open its page on a click. | by reading; screenshot of a hover |
| REQ-004 | A panel in the view shall hold Filters (search text, tags, unresolved, orphans), Groups (a query per group with a colour from the theme's palette), Display (arrows, text fade threshold, node size, link thickness) and Forces (centre, repel, link, link distance), each foldable, with the settings remembered between runs. | screenshot with the panel open; state file round trip |
| REQ-005 | WHEN "Open local graph" is chosen for a page, the app shall show that page's neighbourhood to a chosen depth in a graph tab that follows the open page. | screenshot |
| REQ-006 | Colours shall come from the theme's graph tokens (`graph-line`, `graph-node`, `graph-node-tag`, `graph-node-attachment`) and the group colours from the theme's palette. | screenshot in two themes |

## Scope

- In: the graph tool, the view, the panel, the local graph, the screenshot scene.
- Out: attachments as nodes (the vault's non-markdown files are few and TICKET-002
  shows them in the explorer), saving several group presets, exporting the graph.

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/graph-views.spec.md`
- Related docs: `openwiki/workspace-app.md`, `openwiki/vault-and-brain.md`
- Promoted from intake: `INTAKE-knowledge-workspace` (REQ-008 there)
- Follow-ups opened: a Barnes-Hut pass for large vaults, if they come
