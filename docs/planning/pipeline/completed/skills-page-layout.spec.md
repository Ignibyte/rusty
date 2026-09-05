---
title: Skills page layout
pipeline_id: 2e5afa0a-3edb-46f2-aa18-f2c0a1d69379
status: Phase 5 — Complete PASS
ticket: TICKET-023
ticket_doc: docs/planning/tickets/open/TICKET-023-skills-page-layout.md
aar: docs/planning/knowledge/aar/AAR-023-skills-page-layout.md
sealed:
created: 2026-09-05
---

# Skills page layout: spec

## Intent

The Skills page has a fixed 300px list beside its detail and two sections, Skills and
Scripts, that are always both open. A long skill body has no room, and on a short window
the Scripts list pushes the skill list out of reach. Every other pane in the app resizes;
this one cannot. Chad, 2026-09-04: "the skills page needs a dragging capability on its
panel + expand and hide the skills and scripts".

## Scope

- In: a draggable split between list and detail; Skills and Scripts as collapsible
  sections whose headers toggle them, the open one taking the freed height; the divider
  position and both collapsed states persisted in the workspace state; the window's
  `Splitter` lifted into `Splitter.qml` so the sidebars and this page share one.
- Out (named seams, not forgotten): reordering skills; editing anywhere but the existing
  editor; a splitter for the detail's own body/findings split; per-page state for any
  other page.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the divider between the list and the detail is dragged, the list shall follow the pointer within 200–600 px, using the same `Splitter` the sidebars use. | reading; smoke by Chad |
| REQ-002 | WHEN the Skills or Scripts header is clicked, that section shall collapse or expand, and the open section shall take the freed height. | offscreen screenshot with one section collapsed |
| REQ-003 | WHEN the app restarts, the divider position and each section's state shall be as they were left. | reading of the load/save path; the state file after a change |
| REQ-004 | WHEN the sidebars are dragged after this change, they shall behave exactly as TICKET-022 left them. | reading: same component, call sites unchanged in behaviour |
| REQ-005 | WHEN a section header is focused, Enter or Space shall toggle it. | reading |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | `Splitter` moves from an inline `component` in `Main.qml` to `Splitter.qml` in the module | "The same splitter behaviour" means the same code, not a copy; a second copy would drift from the fix TICKET-022 made | Duplicating the MouseArea in the page |
| 2 | The page reports its layout as one JSON string and receives one back; `ui.skillsLayout` is a single string key | The workspace state already keeps structured page state as strings (`graph`, `bookmarks`, `expanded`); one key means one declare, one load, one save line | Three typed keys |
| 3 | Sections collapse by a header row with a chevron; the header is a focusable, tappable item | Keyboard first (§10): Enter and Space toggle; a bare `Text` would not | A `Button` per header (heavier than the page's other headers) |
| 4 | The split is clamped 200–600 | Below 200 the list's own header row wraps; above 600 the detail's editor loses its width on a 1500px window | No clamp |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-023-skills-page-layout.md`
- Depends on: TICKET-022 (the corrected splitter)
- Architecture: `openwiki/workspace-app.md` (the workspace state; invariant that page state lives in the JSON the Rust side owns)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; no seal |
| 2 Design | Manifest, the state key, the component lift | design actionable |
| 3 Implement | `Splitter.qml`, `build.rs`, `Main.qml`, `SkillsPage.qml` | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger | confirmed findings resolved |
| 4 Validate | Offscreen shots, the state file, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
