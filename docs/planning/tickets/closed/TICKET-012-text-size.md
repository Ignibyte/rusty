---
title: TICKET-012-text-size
status: done
ticket_number: 012
type: feature
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/text-size.spec.md
---

# TICKET-012-text-size

## Summary

A text-size setting for the app's chrome: one base size on the theme that every label
derives from, a stepper in Settings, zoom keys, and a default two pixels larger than today.

## Why

The QML carries about 180 literal pixel sizes from 9 to 22 and the theme has no scale, so
the size cannot be changed without editing every file. Chad finds it small and wants a
setting.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the app starts, every text size in the QML shall derive from one base size on the theme, with no literal pixel size outside the theme. | a test that scans the QML for literal `pixelSize` values and finds none |
| REQ-002 | WHEN the base size is unset, it shall be 14 (today's body size plus two). | unit |
| REQ-003 | WHEN Settings shows "This machine", it shall offer a Text size stepper from 12 to 18 that applies live and is saved in the workspace state. | screenshot; state-file test |
| REQ-004 | WHEN Ctrl and plus, minus or zero are pressed outside a terminal, the base size shall step up, step down or reset. | hotkey test |
| REQ-005 | WHEN the base size changes, the terminal's font size shall not change (it follows the Alacritty font as today). | smoke |
| REQ-006 | WHEN the base size changes, every fixed width or height that holds text (columns, the ribbon, the top bar) shall scale with it. | screenshots at 12 and 18 |

## Scope

- In: the theme's base size and the function every label uses, the sweep of the QML, the Settings stepper, the hotkeys, the workspace-state key.
- Out: terminal zoom; per-page zoom; the reading view's typography scale beyond the base.

## Notes

- Pipeline spec: docs/planning/pipeline/completed/text-size.spec.md
- Related docs: `crates/rusty-app/src/theme.rs`, `crates/rusty-app/qml/SettingsPage.qml`, the workspace state in `crates/rusty-app/src/terminals.rs`.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "lets raise the font size up at least 2-3 more pixels and make a settings for that".
- Follow-ups opened: none.
