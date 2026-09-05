---
title: TICKET-023-skills-page-layout
status: open
ticket_number: 023
type: feature
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/skills-page-layout.spec.md
---

# TICKET-023-skills-page-layout

## Summary

Give the Skills page a draggable split between its list and its detail, and let the Skills and Scripts sections collapse and expand.

## Why

The page has a fixed split and two sections that are always both open. A long skill body has no room, and the Scripts section pushes the skill list out of reach on a short window. Every other pane in the app can be resized; this one cannot.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the divider on the Skills page is dragged, the list and detail shall resize with the pointer within sane bounds, using the same splitter behaviour as the sidebars. | smoke |
| REQ-002 | WHEN the Skills or Scripts section header is clicked, that section shall collapse or expand, and the other shall take the freed height. | smoke; screenshot |
| REQ-003 | WHEN the app restarts, the divider position and each section's collapsed state shall be as they were left. | smoke across a restart |

## Scope

- In: `SkillsPage.qml`; persistence through the same `Settings` the window already uses.
- Out: reordering skills; editing them anywhere but the existing editor.

## Notes

- Depends on TICKET-022's splitter fix — take the corrected component, do not copy the broken one.
- Pipeline spec: TBC.
