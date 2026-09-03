---
title: TICKET-008-amber-phosphor-and-themes
status: done
ticket_number: 008
type: feature
created: 2026-09-03
closed: 2026-09-03
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/amber-phosphor-and-themes.spec.md
---

# TICKET-008-amber-phosphor-and-themes

## Summary

The workspace takes the look of the Replit mock (`docs/design/rusty-omarchy.html`): amber
phosphor on near-black olive, monospace type everywhere, uppercase micro-labels, square
geometry, four accents with roles, and the chrome the mock adds around Obsidian's layout
(a top bar, rail labels, a note meta line, heading marks, code header strips, a status
line, a local-graph legend, an assistant header on the agent pane, a CRT overlay). The
look is a theme: the mock's palette is the default preset, "Follow Omarchy" maps the
desktop theme onto the same roles, and a file in `~/.config/rusty/themes/` is a theme of
the user's own. Settings picks the source and the toggles.

## Why

Chad saw the workspace working on 2026-09-03 and set the next step: design it around the
mock with more colour, and make it themeable. The mock's identity is stronger than a
recoloured Obsidian; the roles it gives its colours (amber for the active thing, gold for
titles, teal for what is alive, red for errors) are what make it read.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The app shall paint every surface from one token set with the mock's roles (background, three panel levels, line and bright line, muted, text, bright, accent, soft accent, gold, alive, red, plus the derived hover, active, selection and code colours), and no colour shall be written into QML outside that set. | `grep` for hex colours in QML; screenshots |
| REQ-002 | The token set shall come from one of three sources chosen in Settings: a built-in preset (Amber phosphor from the mock, the default, and at least two more), "Follow Omarchy" (the desktop theme's palette mapped onto the roles, re-read on `omarchy theme set`), or a TOML file in `~/.config/rusty/themes/`; the choice and the toggles shall survive restarts. | screenshots per source; a theme file round trip; state file by reading |
| REQ-003 | The app shall set its type from the theme: the mock's monospace face at the mock's sizes for every label, and the terminal font for terminals; the micro-labels shall be uppercase with the mock's letter spacing. | screenshots |
| REQ-004 | The window shall carry the mock's chrome: a top bar (brand, Hyprland workspace numbers with the active one lit, the vault's state and page count, memory, CPU, the clock), rail buttons with labels, pane headers as micro-labels with counts, tree rows with the active bar and folder counts, tabs with glyphs and an amber underline, a breadcrumb bar with the read toggle, the status line under the tree, the command layer with the mock's border and glow, a toast in the mock's style, and a CRT overlay switched in Settings. | screenshots; the workspace strip by reading (no Hyprland offscreen) |
| REQ-005 | A page shall show the mock's note furniture: the meta line (live dot, modified time, backlink count), the title with its amber `#`, section titles with `##` and a rule, callouts with the amber bar and tint, code blocks with a header strip, square task boxes with a teal check, and a "LINKED FROM" footer; the renderer shall take these from the theme's style. | renderer tests; screenshots |
| REQ-006 | A page shall show a local-graph legend card (node count, direct links, related, distant) that opens the local graph on a click. | screenshot; by reading |
| REQ-007 | The right pane's agent view shall wear the mock's assistant header (sigil, name, READY state) and a context card naming the open page and its backlink count, above the terminal; the other panes shall wear the same header style. | screenshots |
| REQ-008 | Every existing screenshot scene shall render in the default preset, and one scene each in "Follow Omarchy" and a theme file. | `scripts/screenshot.sh` |

## Scope

- In: the token model and its three sources, the Settings picker and toggles, the
  application font, every QML surface restyled, the renderer's style additions, the top
  bar's system readings, the legend card, the assistant header, the CRT overlay, the
  screenshot scenes, docs and wiki.
- Out: a chat pane that talks to a model (the mock's assistant conversation; a later
  ticket with a provider setting), Hyprland workspace switching from the strip beyond
  what `hyprctl` gives, syntax colouring inside the reading view's code blocks, theme
  editing inside the app.

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/amber-phosphor-and-themes.spec.md`
- Design references: `docs/design/rusty-omarchy.html` (rendered with headless Chromium
  for the record), Chad's answers of 2026-09-03
- Related docs: `openwiki/workspace-app.md`, `openwiki/markdown-rendering.md`
- Follow-ups opened: a model-backed chat pane (the mock's conversation); in-app theme editing; the section rule beside the title if rich text ever allows it
