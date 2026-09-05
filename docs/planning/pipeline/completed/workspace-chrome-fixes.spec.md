---
title: Workspace chrome fixes
pipeline_id: e8f444c2-71de-4e14-8e1a-fff6b5c7ca14
status: Phase 5 — Complete PASS
ticket: TICKET-022
ticket_doc: docs/planning/tickets/open/TICKET-022-workspace-chrome-fixes.md
aar: docs/planning/knowledge/aar/AAR-022-workspace-chrome-fixes.md
sealed:
created: 2026-09-05
---

# Workspace chrome fixes: spec

## Intent

Four things wrong with the workspace chrome, all in `Main.qml`, all found by Chad using
the app on 2026-09-04: the sidebar splitters do not follow the pointer; tabs cannot be
dragged to reorder although the Settings page says they can; the tab rename field
overflows its dialog to the right; and the tab strip's `+` opens only the page switcher
when the hand that reaches it as often wants an agent.

## Scope

- In: `Splitter` (both sidebars), the tab delegate and `moveTab`, `renameDialog`, the
  strip's `+`, and the one sentence in `SettingsPage.qml` that describes reordering.
- Out (named seams, not forgotten): detaching a tab into its own window; drag between
  windows; drag of the ribbon or the sidebar panes; the Skills page's own splitter
  (TICKET-023 takes the corrected component).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN either sidebar splitter is dragged, the pane shall follow the pointer for the whole drag, including when the pointer leaves the handle, within the existing minimum and maximum widths. | reading of the coordinate frame; smoke by Chad |
| REQ-002 | WHEN a document tab is dragged along the strip and released, it shall move to the drop position through `moveTab`, so the order persists as it already does. | reading; smoke by Chad |
| REQ-003 | WHEN a tab is renamed, the field shall sit inside the dialog's width rather than extend past it. | offscreen screenshot of the open dialog |
| REQ-004 | WHEN the strip's `+` is used, it shall offer the page switcher and the same agents the top bar offers, and choosing an agent shall open it in a new terminal tab. | reading; offscreen screenshot of the open menu; smoke by Chad |
| REQ-005 | WHEN the Settings page describes reordering, its text shall match what the app does. | reading |
| REQ-006 | WHEN a tab is clicked, middle-clicked or right-clicked, it shall still select, close, or open its menu as before; a drag shall not fire a select. | reading: the handlers stay, the drag handler owns only movement |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The splitter measures in scene coordinates: `mapToItem(null, mouse.x, 0).x` at press and at every move | `mouse.x` is in the handle's own frame; the drag moves the handle, which re-bases the origin every event and is exactly the bug | A `DragHandler` on the handle (fine, but the existing MouseArea with a fixed frame is the smaller change and keeps the cursor shape logic) |
| 2 | Tab drag copies `TasksPage.qml`'s `DragHandler`: `target: null`, one axis, `onActiveChanged` computing the drop index from the centroid | It is the working pattern in this codebase, and it does not fight the strip's other handlers because a `DragHandler` only activates past the drag threshold | A `Drag`/`DropArea` pair, which needs a proxy item and a visual the strip does not have |
| 3 | The drop index is the tab under the centroid's x in `tabRow`, found by walking the row's children | The row is a `Row` of fixed-width delegates, not a `ListView`, so there is no `indexAt`; a walk is four lines | Converting the strip to a `ListView` (out of proportion) |
| 4 | `renameDialog` gets the same shape as `newTabDialog`: a `ColumnLayout` with `Layout.preferredWidth: 320` | A bare child with an explicit `width` does not size a `Dialog`'s content; the working dialog beside it already shows the shape that does | Setting the dialog's own width |
| 5 | The `+` opens a `Menu`: the page switcher (Ctrl+T) first, one item per agent in `win.agents` opening a terminal tab, then "Custom terminal…" (Ctrl+Shift+T) | "As well" means keep the switcher and add the agents; a menu on a plus is what people expect, and it is discoverable where a right-click would not be; Ctrl+T itself is unchanged | Left click switcher, right click agents (undiscoverable); a second button |
| 6 | The Settings sentence is corrected in the same change | It is the app describing itself wrongly; it becomes true the moment REQ-002 lands and must say so | Leaving it for a docs pass |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-022-workspace-chrome-fixes.md`
- Intake: none; reported from use
- Architecture: `openwiki/workspace-app.md` (layout, tabs, the workspace state)
- Pattern copied: `crates/rusty-app/qml/TasksPage.qml` (the drag handle)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; no seal needed |
| 2 Design | Manifest, the four mechanisms, regression table | design actionable |
| 3 Implement | The edits to `Main.qml` and `SettingsPage.qml` | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger | confirmed findings resolved |
| 4 Validate | Offscreen screenshots of the dialog and the menu; `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
