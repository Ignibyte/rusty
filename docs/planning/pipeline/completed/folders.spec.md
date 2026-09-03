---
title: Folders, part one
pipeline_id: bb302c74-c180-4ce4-aea4-68a6ff889539
status: Phase 5 — Complete PASS
ticket: TICKET-016
ticket_doc: docs/planning/tickets/closed/TICKET-016-folders.md
aar: docs/planning/knowledge/aar/AAR-016-folders.md
sealed: Chad, 2026-09-03 17:20, in the rustal session (relayed): "lets run them all including 010", with the rustal session's recommendations as the answers (any folder as a root below the vault, remembered per machine and removable; markdown as a rendered tab, text as a monospace viewer with line numbers, images as an image tab, everything else through the desktop; a right-click on a folder offers one entry per agent that opens a terminal tab there, plus copy path and reveal; links, backlinks, graph and search stay vault-only; file operations and git decorations are parts two and three)
created: 2026-09-03
---

# Folders, part one: spec

## Intent

The left pane grows from the vault tree to any folder on the machine: roots below the
vault, remembered per machine and removable; files viewed by kind; "open an agent here"
on a folder. Chad, 2026-09-03 15:40: "we should look at building a full blown file
explorer as well". Part one of three; parts two (file operations) and three (git
decorations) are minted here as tickets and built later.

## Scope

- In: REQ-001, REQ-002, REQ-003 and REQ-006 of the ticket: the roots and their
  persistence, the three viewer kinds, the folder menu with the agents, copy path and
  reveal, the vault-only boundary for links, backlinks, graph and search.
- Out (named seams): new file, new folder, rename, move, delete on disk (part two,
  `TICKET-019`); git decorations (part three, `TICKET-020`); editing a file outside the
  vault (with part two); a watcher on the roots (a Refresh entry stands in); a file manager
  for the whole system.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN "Add folder" is chosen in the left pane, the chosen folder shall appear as a root below the vault tree, remembered per machine in the workspace state under `roots`, and removable from its menu. | the `folders` scene; the state key by reading |
| REQ-002 | WHEN a file under a root is clicked, markdown shall open rendered like a page (with a Source toggle), text shall open in a monospace viewer with line numbers, an image shall open fitted in a tab, and anything else shall open through the desktop's handler. | unit tests on the kind and the listing; the scene |
| REQ-003 | WHEN a folder under a root is right-clicked, the menu shall offer one entry per agent found on the machine that opens a terminal tab with that folder as its working directory, a shell there, copy path and reveal in the file manager. | reading; the terminal's `cwd` path exists since TICKET-002 |
| REQ-006 | WHEN a root is shown, links, backlinks, graph and search shall stay vault-only. | review: no root reaches a brain tool |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The disk is read by the app through a small Rust type (`Folders`: list, kind, text, base name, open externally), not through the back end. | The disk is not the store; `AD-rusty-mcp-only-back-end-001` guards the store. Listing a folder through an MCP round trip would make every expansion asynchronous for no gain. | A `folder_list` tool. |
| 2 | Disk rows join the explorer's one list with their own kinds (`root`, `dir`, `disk`) so scrolling, keys and the current row stay one thing. | A second list would split the pane's height. | Two lists. |
| 3 | A markdown file renders through `brain_render` given its text (an additive `markdown` parameter), read-only, with a Source toggle. | The renderer already knows the theme's rich text; a file outside the vault is not a page and gets no page tools. | A page tab (would need a slug). |
| 4 | Roots live under `roots` in the workspace state (`[{path, name}]`) and their expansion under the same `expanded` map keyed by absolute path. | The state file is per machine already. | A setting in the store (would travel). |
| 5 | Hidden entries (a leading dot) are skipped; folders sort first, then names without case; a listing is cached until Refresh or a root change. | What Obsidian and most explorers do; part one has no watcher. | Showing dotfiles. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-016-folders.md`
- Intake: none
- Design references: `crates/rusty-app/qml/Explorer.qml`, `Main.qml` (the tab model,
  `openTerminal(program, name, session, cwd)`, the `ui` state), `AgentTerminal.qml`
  (`cwd` → `startDir`), `src/desk.rs` (the cxx-qt bridge pattern), `src/terminals.rs`
  (the state file), `RenderParams` in `crates/rusty-mcp/src/main.rs`
- Architecture: `AD-rusty-mcp-only-back-end-001`, `AD-rusty-workspace-is-obsidian-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; sealed |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
