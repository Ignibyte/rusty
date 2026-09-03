---
title: TICKET-016-folders
status: done
ticket_number: 016
type: feature
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/folders.spec.md
---

# TICKET-016-folders

## Summary

Folders: the left pane grows from the vault tree to any folder on the machine, with viewing,
file operations and "open an agent here". Part one of three: roots, viewing and the agent
entry; parts two and three are minted at design.

## Why

The explorer knows only the vault, and Rusty's agents work in repositories. A folder tree
beside an agent terminal is the note-and-agent pane pattern applied to code, and the entry
that opens Claude Code in a chosen folder is what makes a file explorer worth building here.
Chad asked for a full one.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN "Add folder…" is chosen in the left pane, the chosen folder shall appear as a root below the vault, remembered per machine in the workspace state, and removable. | state test; screenshot |
| REQ-002 | WHEN a file in a folder root is clicked, markdown shall open as a page tab, text as a monospace viewer tab with line numbers, an image as an image tab, and anything else through the desktop (`xdg-open`). | tests per kind |
| REQ-003 | WHEN a folder is right-clicked, the menu shall offer one entry per agent found on the machine that opens a terminal tab with that folder as the working directory, plus copy path and reveal in the file manager. | terminal working-directory test; smoke |
| REQ-004 | (part two) WHEN new file, new folder, rename, move by drag and delete are used in a folder root, they shall act on the disk, and delete shall move to the trash. | tests on a temporary tree |
| REQ-005 | (part three) WHEN a folder root is a git repository, the tree shall decorate modified, added and untracked files. | test on a temporary repository |
| REQ-006 | WHEN a folder root is shown, links, backlinks, graph and search shall stay vault-only. | review |

## Scope

- In: REQ-001 to REQ-003 and REQ-006; the viewer tabs; the per-machine root list.
- Out: REQ-004 and REQ-005, which become their own tickets at design; a file manager for the whole system (Nautilus is there); permissions editing.

## Notes

- Pipeline spec: docs/planning/pipeline/active/folders.spec.md
- Related docs: `crates/rusty-app/qml/Explorer.qml`, `AgentTerminal.qml`, `crates/rusty-app/src/terminals.rs` (the workspace state).
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "we should look at building a full blown file explorer as well".
- Delivered on 2026-09-03 (part one: REQ-001, REQ-002, REQ-003, REQ-006). Sealed by
  Chad at 17:20 through the rustal session ("lets run them all including 010").
- Follow-ups opened: TICKET-019 (file operations, REQ-004) and TICKET-020 (git
  decorations, REQ-005), minted at design; a large folder lists synchronously, for
  part two to page.
