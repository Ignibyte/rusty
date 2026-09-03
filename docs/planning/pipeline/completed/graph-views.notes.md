---
title: Graph views: notes
pipeline_id: 9e4b7f12-3c8a-4d6e-b1f5-7a2c9d0e4f63
---

# Graph views: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-tags-one-index-001` (tags and their counts),
  `AD-rusty-lenient-pages-001`, `PR-rusty-qml-component-scope-001`,
  `PR-rusty-workspace-state-in-json-001`, `PR-rusty-lazy-pane-terminals-001`; completed
  notes of TICKET-002 (tab kinds, the state file, screenshots) and TICKET-003 (`tag:`
  search, the Tags pane); wiki `workspace-app.md` (tab kinds, extension points),
  `vault-and-brain.md` (`brain_links` rows hold the resolved slug or the raw target,
  `brain_tags`); the brain's `projects/rusty-v3` timeline. The theme tokens already
  carry `graph-line`, `graph-node`, `graph-node-tag` and `graph-node-attachment` from
  the Omarchy theme's `obsidian.css`.
- Decisions: the five in the spec.
- Seal: Chad's goal of 2026-09-02.

## Phase 2: Design

- Architecture and data flow: `BrainManager::graph(options)` reads `brain_pages`,
  `brain_links` (a left join tells resolved from unresolved) and `brain_tags`, builds
  `Graph { nodes, edges }` (node `id` is the slug, `tag:<name>` or `new:<target>`,
  with `kind` page, tag or unresolved, `title`, `page_type`, `folder`), and when
  `around` is set keeps the breadth-first neighbourhood to `depth`. The tool
  `brain_graph` passes the options through. The app opens a `graph` tab kind
  (`GraphView.qml`) that fetches the graph, runs a force simulation on a timer
  (repulsion between every pair, springs on edges toward the link distance, a centre
  pull, damping), and draws on a `Canvas` with pan and zoom; hover finds the nearest
  node; a click opens the page; a drag pins a node while held. The panel folds into
  Filters, Groups, Display and Forces; its values live under `graph` in the workspace
  state. The local graph is the same view with `around` set to the tab's page; while
  the tab is current, it follows the last page the window opened.
- File manifest:

| File | Purpose |
|---|---|
| `crates/rusty-core/src/brain/mod.rs` | `GraphOptions`, `GraphNode`, `GraphEdge`, `Graph`, `graph()` |
| `crates/rusty-mcp/src/main.rs` | `brain_graph`, router list; smoke |
| `crates/rusty-app/qml/GraphView.qml` | the view: data, forces, canvas, interaction, panel |
| `crates/rusty-app/qml/Main.qml` | the `graph` tab kind, ribbon button, Ctrl+G, palette commands, state |
| `crates/rusty-app/qml/NoteTab.qml` | "Open local graph" in the page menu |
| `crates/rusty-app/build.rs` | the new QML file |
| `scripts/screenshot.sh` | `graph` scenes |
| `README.md`, `ROADMAP.md`, `openwiki/` | the tier |

- Store consequences: none.
- Tool contract: `brain_graph` (`tags`, `unresolved`, `around`, `depth`, all optional)
  returns `{nodes: [{id, kind, title, page_type, folder}], edges: [{from, to}]}`.
- Regression plan: REQ-001 core tests (edges from resolved links, tag and unresolved
  nodes on request, the neighbourhood by depth), smoke; REQ-002 to REQ-006 screenshots
  (global, local, the panel, a hover, a light theme) and reading.
- Risks: the simulation's cost grows with the square of the node count (a few hundred
  is fine; a timer stops when the layout settles); the `Canvas` needs repaints only on
  ticks and interaction; the graph tab must not hold keys the editor uses.
- CodeGraph evidence: `get_all_links` (no callers besides tests), `tags` (the tool),
  `list_pages` (the switcher, `pages_in`).

## Phase 3: Implement

- Built: the manifest. `GraphOptions`, `GraphNode` (with the page's tags for group
  queries), `GraphEdge`, `Graph` and `BrainManager::graph` (pages, resolved links, tag
  and unresolved nodes on request, a breadth-first neighbourhood for `around` and
  `depth`); `brain_graph` and the router list; the smoke test asks for a local graph.
  `GraphView.qml`: the data, a force simulation on a 33 ms timer with cooling and a
  settle check, the canvas (edges, arrows, dots by degree, labels by zoom and hover,
  hover dimming), pan, zoom around the cursor, node drag with pinning, click to open,
  the panel (Filters with a depth slider for the local graph, Groups with palette
  swatches, Display, Forces), settings persisted through the window's state file. The
  `graph` tab kind (global as one tab, local as one tab that follows the last page),
  the ribbon button, Ctrl+G, two palette commands, "Open local graph" in the page menu,
  a graph glyph in the tab strip, `graph` and `localgraph` scenes and a denser fixture
  in the screenshot script.
- Deviations: none from the manifest. The screenshot script gained a fix on the way:
  its fixture wrote into a folder it never created, and the cleanup trap ran
  `kill "${server:-0}"` on an early exit, which is `kill 0`, the whole process group.
- Fast gate: `cargo test -p rusty-core` 235 + 7 passed; `cargo test -p rusty-mcp` 3 + 1
  passed; clippy in the gate.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | `signal settingsChanged(var)` on a component with `property var settings` clashes with the property's own change signal; the whole window failed to load | high | fixed (`settingsEdited`); the log path from AAR-002 found it |
| 2 | correctness | a signal handler with a bare `for` statement (`onChildrenChanged: for ...`) fails qmlcachegen | low | fixed (braces) |
| 3 | correctness | the screenshot fixture's `meetings/` folder was never created, so the script died before the server started | medium | fixed |
| 4 | data safety | the cleanup trap's `kill "${server:-0}"` became `kill 0` on an early exit and killed the calling shell and its pipeline | high | fixed: kill only when the pid is set (PR-rusty-never-kill-zero-001) |
| 5 | SQLite lock | `graph()` reads three statements under one scoped guard and builds outside it | ok | verified by reading |
| 6 | keyboard | Ctrl+G joins the workspace keys and stands down while a terminal has focus; the graph tab holds no other keys | ok | by reading |
| 7 | theme | node, tag and line colours come from the theme's graph tokens; group colours from the palette; the panel from the surface tokens | ok | screenshots |
| 8 | performance | the simulation is quadratic in the node count; a timer stops when the layout settles and restarts on interaction; a few hundred pages is the expected size | note | accepted; a Barnes-Hut pass is a later tidy-up if vaults grow |
| 9 | prose | docs, the wiki pages and tool descriptions against `no-ai-slop` | ok | clean |

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-core`: 235 passed, 7
  integration passed (`graph_nodes_edges_and_neighbourhoods` among them). `cargo test -p
  rusty-mcp`: 3 passed; `tests/smoke.rs` 1 passed (a local graph around a page lists its
  neighbour and the edge).
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff] (receipt 2026-09-03T05:26:59Z).
- Smoke evidence: `scripts/screenshot.sh` scenes `graph` (eleven nodes laid out, the
  panel open) and `localgraph` (nine nodes around the page, the depth slider); the first
  is `docs/screenshots/graph-view.png`.
- Skips or pre-existing failures: hover, drag, zoom and click are verified by reading
  the handlers; no synthetic input on the desktop.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (core test, smoke); REQ-002 PASS (screenshots, the
  simulation and canvas by reading); REQ-003 PASS (handlers by reading; the hover
  dimming and tooltip in code); REQ-004 PASS (screenshot with the panel; the settings
  round trip through the workspace state by reading); REQ-005 PASS (local graph
  screenshot with the depth slider; follows `lastPageSlug`); REQ-006 PASS (tokens by
  reading; the dark screenshot).
- Wiki: `update` run through the lifecycle, openwiki_finish returned status complete (receipt 2026-09-03T05:29:04Z); `workspace-app` and
  `mcp-back-end` reconciled with the graph, and the back end page's tools claim restored.
- Docs: README (the graph, the tool, the screenshot), ROADMAP (M8 line), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-004-graph-views.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | a signal named after a property's change signal | PR-rusty-qml-signal-names-001 |
| 2026-09-03 | `kill "${var:-0}"` in a trap | PR-rusty-never-kill-zero-001 |
| 2026-09-03 | the graph lives in the app on a canvas | AD-rusty-graph-in-the-app-001 |
