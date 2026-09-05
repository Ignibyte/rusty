---
title: TICKET-022-workspace-chrome-fixes
status: done
ticket_number: 022
type: fix
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/workspace-chrome-fixes.spec.md
---

# TICKET-022-workspace-chrome-fixes

## Summary

Four things wrong with the workspace chrome: the sidebar splitters do not track the cursor, tabs cannot be dragged to reorder though the app says they can, the tab rename field overflows to the right, and the tab strip's `+` cannot start an agent.

## Why

All four are daily friction in `Main.qml`, and two of them are the app contradicting itself.

**The splitters.** `Splitter` computes its delta from `mouse.x`, which is in the MouseArea's own coordinate space. Dragging changes `ui.leftWidth`, which moves the splitter, which re-bases `mouse.x` under the cursor on the next event — so the handle only tracks while the pointer stays inside its 7px strip and the pane appears to fight the drag.

**Tab drag.** `moveTab(from, to)` exists and is bound to `Ctrl+Shift+PgUp/PgDown` and the right-click menu, but there is no `DragHandler` anywhere in `Main.qml`. Meanwhile `SettingsPage.qml` tells the user "drag a tab or a task's handle to reorder". Tasks have one (`TasksPage.qml`); tabs never got one.

**The `+`.** The top bar can start an agent in a tab, but the tab strip's `+` cannot, which is where the hand already is.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN either sidebar splitter is dragged, the pane shall follow the pointer for the whole drag, including when the pointer leaves the handle, within the existing min and max widths. | smoke: drag from 180 to 600 in one motion |
| REQ-002 | WHEN a document tab is dragged along the tab strip, it shall reorder to the drop position, and the reorder shall persist the way `moveTab` already persists. | smoke; screenshot |
| REQ-003 | WHEN a tab is renamed, the rename field shall stay within the tab's width and not extend past the strip. | screenshot |
| REQ-004 | WHEN the tab strip's `+` is used, it shall offer the same agents the top bar offers, and starting one shall open it in a new terminal tab. | smoke |
| REQ-005 | WHEN the Settings page describes reordering, its text shall match what the app does. | reading |

## Scope

- In: `Main.qml` (`Splitter`, the tab delegate, the tab strip `+`), the rename field, `SettingsPage.qml`'s help text.
- Out: detaching a tab into its own window; drag between windows; the ribbon.

## Notes

- REQ-001: map to a fixed frame (`mapToItem(null, mouse.x, 0).x`) or use a `DragHandler` on the x axis. Both splitters, left and right.
- REQ-002: `TasksPage.qml`'s drag handle is the working pattern in this codebase — copy its shape rather than inventing one.
- Pipeline spec: TBC.
