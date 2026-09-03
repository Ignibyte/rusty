---
title: Text size
pipeline_id: c796967f-73d9-4675-9107-40393f31f132
status: Phase 5 — Complete PASS
ticket: TICKET-012
ticket_doc: docs/planning/tickets/open/TICKET-012-text-size.md
aar: docs/planning/knowledge/aar/AAR-012-text-size.md
sealed: not required (a setting, no new tab, store or dependency). Direction: Chad, 2026-09-03 15:40, "lets raise the font size up at least 2-3 more pixels and make a settings for that"; 16:00, "lets start working on these" (relayed by the rustal session)
created: 2026-09-03
---

# Text size: spec

## Intent

One base text size on the theme that every label in the app derives from, a Text size
stepper in Settings, Ctrl with plus, minus and zero, and a default two pixels larger than
today's body size. The reading view follows the same base through the render style.
Chad finds the type small and wants it to be a setting.

## Scope

- In: `baseSize` and `scale` on `Theme`, the sweep of every literal text size in the QML
  onto the scale, the chrome heights that hold text, the render style's `size`, the
  Settings stepper, the three keys and their palette commands, the workspace-state key
  `textSize`, the `RUSTY_TEXT_SIZE` override for screenshots and tests, a test that
  refuses a literal size in the QML, docs and wiki.
- Out (named seams, not forgotten): terminal zoom (the terminal keeps the Alacritty
  font); per-page zoom; a typographic scale for the reading view beyond its base size.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the app starts, every text size in the QML shall derive from one base size on the theme, with no literal pixel size outside the theme. | a test that scans the QML for literal `pixelSize` values and finds none |
| REQ-002 | WHEN the base size is unset, it shall be 14 (today's body size plus two). | unit |
| REQ-003 | WHEN Settings shows "This machine", it shall offer a Text size stepper from 12 to 18 that applies live and is saved in the workspace state. | screenshot; state-file test |
| REQ-004 | WHEN Ctrl and plus, minus or zero are pressed outside a terminal, the base size shall step up, step down or reset. | hotkey test |
| REQ-005 | WHEN the base size changes, the terminal's font size shall not change (it follows the Alacritty font as today). | smoke |
| REQ-006 | WHEN the base size changes, every fixed width or height that holds text (columns, the ribbon, the top bar) shall scale with it. | screenshots at 12 and 18 |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The theme carries `baseSize` (12 to 18, default 14) and `scale` (`baseSize / 12`), and every QML size is `Math.round(n * theme.scale)` where `n` is today's pixel value. | A property read inside a binding is tracked by the engine, so the whole window re-lays out live; a Rust invokable would not be. The sweep keeps every proportion the mock set. | A type scale of named steps (would round twelve distinct sizes into seven and change the look); a `px()` invokable on the Rust object (no dependency tracking). |
| 2 | The reading view takes its base from the same scale through the render style's `size`. | The body text is what Chad reads most. | Leaving the reading view at its fixed size. |
| 3 | The terminal keeps `pointSize: 11` on the Alacritty font; the source editors scale. | REQ-005; the editors are chrome that holds text. | Scaling the terminal (a separate zoom later). |
| 4 | The size lives in the workspace state as `textSize`; `RUSTY_TEXT_SIZE` overrides it for screenshots and tests. | The same JSON file the skin lives in; the screenshot script needs a knob (PR-rusty-workspace-state-in-json-001). | A back-end setting (it is per machine, per eye). |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-012-text-size.md`
- Intake: none
- Design references: `crates/rusty-app/src/theme.rs`, `crates/rusty-app/qml/SettingsPage.qml`,
  `crates/rusty-core/src/brain/render.rs` (`Style::size`), the workspace state in
  `crates/rusty-app/qml/Main.qml`
- Architecture: `docs/architecture.md` (the skinned bullet: `Theme` turns the roles into
  every token the QML binds to)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
