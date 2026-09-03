---
title: Amber phosphor and themes
pipeline_id: c2bb8a6f-7da2-489c-87ed-d30218b1b8fe
status: Phase 5 — Complete PASS
ticket: TICKET-008
ticket_doc: docs/planning/tickets/closed/TICKET-008-amber-phosphor-and-themes.md
aar: docs/planning/knowledge/aar/AAR-008-amber-phosphor-and-themes.md
sealed: 2026-09-03, Chad: "the only thing that i would like to do next is looking at that html file and we design it around that with more color. i'd like to be able to potentially theme it as well"; answers the same day: default Amber phosphor, all of the chrome, dress the agent pane
created: 2026-09-03
---

# Amber phosphor and themes: spec

## Intent

The workspace looks like the mock and can be re-skinned: one token set with roles, three
sources for it, and the mock's chrome and note furniture on top of Obsidian's layout.

## Scope

- In: `theme.rs` and `omarchy.rs` (roles, presets, the Omarchy mapping, theme files, the
  selection and toggles), the application font, a `Desk` object for the top bar's
  readings, every QML file, the renderer's `Style`, `SettingsPage` (the picker), the
  screenshot scenes, docs and wiki.
- Out (named seams): a model-backed chat pane; code syntax colours in reading view;
  in-app theme editing.

## Acceptance criteria (EARS)

REQ-001 to REQ-008 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | One token set with the mock's roles is the only way colour reaches QML; the Omarchy mapping, the presets and the files all produce it. | One place to reason about contrast and roles; a theme is data. | per-source QML branches |
| 2 | The choice (source, name, toggles) lives in the workspace state and reaches the Rust `Theme` through one invokable; "Follow Omarchy" keeps the watcher on `~/.config/omarchy/current`. | The state file already round-trips; the Rust side owns the desktop reads. | a setting in the back end |
| 3 | The mock's Amber phosphor is the default; Follow Omarchy is a choice. | Chad's answer of 2026-09-03. | Follow Omarchy by default |
| 4 | Monospace everywhere from the theme's face (JetBrains Mono, which the terminal already uses), set as the application font once, with the mock's sizes as tokens. | The mock's type is half its identity; one application font beats a hundred `font.family` lines. | per-item fonts |
| 5 | The top bar's workspace strip reads Hyprland through `hyprctl -j` on a timer and switches on a click; offscreen it shows the mock's static strip. | Rusty is for Omarchy; the numbers mean something there. | decorative numbers |
| 6 | The note furniture that is markup (heading marks, callouts, code header strips, task boxes) comes from the renderer's `Style`, so the app and any agent that renders get the same look. | The renderer already owns the page's markup. | QML post-processing |

## Linked artifacts

- Ticket: TICKET-008
- Design references: `docs/design/rusty-omarchy.html`
- Architecture: `openwiki/workspace-app.md`, `openwiki/markdown-rendering.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | sealed by Chad's words of 2026-09-03 |
| 2 Design | token roles, sources, manifest, regression table in the notes | design actionable |
| 3 Implement | tokens and sources, font, chrome, renderer, panes, legend, overlay | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger | confirmed findings resolved |
| 4 Validate | tests, screenshots in three sources, `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, wiki update, docs, AAR, register, brain capture, archive | pair archived |
